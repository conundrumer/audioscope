use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use portaudio::{
    self,
    PortAudio,
    Stream,
    NonBlocking,
    Input,
    StreamParameters,
    InputStreamSettings,
    InputStreamCallbackArgs,
    Continue,
};
use num::complex::Complex;
use rustfft::algorithm::Radix4;
use rustfft::FFT;

use config::Config;
use display::Vec4;

pub type MultiBuffer = Arc<Vec<Mutex<AudioBuffer>>>;
pub type PortAudioStream = Stream<NonBlocking, Input<f32>>;

pub struct AudioBuffer {
    pub rendered: bool,
    pub analytic: Vec<Vec4>,
}

const SAMPLE_RATE: f64 = 44_100.0;
const CHANNELS: i32 = 2;
const INTERLEAVED: bool = true;

pub fn init_audio(config: &Config) -> Result<(PortAudioStream, MultiBuffer), portaudio::Error> {
    let buffer_size = config.audio.buffer_size as usize;
    let fft_size = config.audio.fft_size as usize;
    let num_buffers = config.audio.num_buffers;
    let cutoff = config.audio.cutoff;
    let q = config.audio.q;

    let pa = PortAudio::new()?;

    let def_input = pa.default_input_device()?;
    let input_info = pa.device_info(def_input)?;
    println!("Default input device name: {}", input_info.name);

    let latency = input_info.default_low_input_latency;
    let input_params = StreamParameters::<f32>::new(def_input, CHANNELS, INTERLEAVED, latency);

    pa.is_input_format_supported(input_params, SAMPLE_RATE)?;
    let settings = InputStreamSettings::new(input_params, SAMPLE_RATE, buffer_size as u32);

    let mut buffers = Vec::with_capacity(num_buffers);

    for _ in 0..num_buffers {
        buffers.push(Mutex::new(AudioBuffer {
            rendered: true,
            analytic: vec![Vec4 {vec: [0.0, 0.0, 0.0, 0.0]}; buffer_size + 3],
        }));
    }
    let buffers = Arc::new(buffers);

    let (receiver, callback) = {
        let mut buffer_index = 0;
        let (sender, receiver) = mpsc::channel();
        let print_drop = config.debug.print_drop;
        let gain = config.audio.gain;
        let buffers = buffers.clone();
        let mut analytic_buffer = vec![Vec4 {vec: [0.0, 0.0, 0.0, 0.0]}; buffer_size + 3];

        let mut time_index = 0;
        let mut time_ring_buffer = vec![Complex::new(0.0, 0.0); 2 * fft_size];
        // this gets multiplied to convolve stuff
        let mut complex_freq_buffer = vec![Complex::new(0.0f32, 0.0); fft_size];
        let mut complex_analytic_buffer = vec![Complex::new(0.0f32, 0.0); fft_size];

        let mut n = fft_size - buffer_size;
        if n % 2 == 0 {
            n -= 1;
        }
        let analytic = make_analytic(n, fft_size);
        let fft = Radix4::new(fft_size, false);
        let ifft = Radix4::new(fft_size, true);

        let mut prev_input = Complex::new(0.0, 0.0); // sample n-1
        let mut prev_diff = Complex::new(0.0, 0.0); // sample n-1 - sample n-2
        let mut angle_lp = get_lowpass(cutoff, q);
        let mut noise_lp = get_lowpass(0.05, 0.7);

        (receiver, move |InputStreamCallbackArgs { buffer: data, .. }| {
            {
                let (left, right) = time_ring_buffer.split_at_mut(fft_size);
                for ((x, t0), t1) in data.chunks(CHANNELS as usize)
                    .zip(left[time_index..(time_index + buffer_size)].iter_mut())
                    .zip(right[time_index..(time_index + buffer_size)].iter_mut())
                {
                    let mono = Complex::new(gain * (x[0] + x[1]) / 2.0, 0.0);
                    *t0 = mono;
                    *t1 = mono;
                }
            }
            time_index = (time_index + buffer_size as usize) % fft_size;
            fft.process(&mut time_ring_buffer[time_index..time_index + fft_size], &mut complex_freq_buffer[..]);

            for (x, y) in analytic.iter().zip(complex_freq_buffer.iter_mut()) {
                *y = *x * *y;
            }
            ifft.process(&mut complex_freq_buffer[..], &mut complex_analytic_buffer[..]);

            analytic_buffer[0] = analytic_buffer[buffer_size];
            analytic_buffer[1] = analytic_buffer[buffer_size + 1];
            analytic_buffer[2] = analytic_buffer[buffer_size + 2];
            let scale = fft_size as f32;
            for (&x, y) in complex_analytic_buffer[fft_size - buffer_size..].iter().zip(analytic_buffer[3..].iter_mut()) {
                let diff = x - prev_input; // vector
                prev_input = x;

                let angle = get_angle(diff, prev_diff).abs().log2().max(-1.0e12); // angular velocity (with processing)
                prev_diff = diff;

                let output = angle_lp(angle);

                *y = Vec4 { vec: [
                    x.re / scale,
                    x.im / scale,
                    output.exp2(), // smoothed angular velocity
                    noise_lp((angle - output).abs()), // average angular noise
                ]};
            }

            let dropped = {
                let mut buffer = buffers[buffer_index].lock().unwrap();
                let rendered = buffer.rendered;
                buffer.analytic.copy_from_slice(&analytic_buffer);
                buffer.rendered = false;
                !rendered
            };
            buffer_index = (buffer_index + 1) % num_buffers;
            if dropped && print_drop {
                sender.send(()).ok();
            }
            Continue
        })
    };
    if config.debug.print_drop {
        use std::io::{self, Write};
        thread::spawn(move || {
            while let Ok(_) = receiver.recv() {
                print!("!");
                io::stdout().flush().unwrap();
            }
        });
    }
    let stream = pa.open_non_blocking_stream(settings, callback)?;

    Ok((stream, buffers))
}

// angle between two complex numbers
// scales into [0, 0.5]
fn get_angle(v: Complex<f32>, u: Complex<f32>) -> f32 {
    // 2 atan2(len(len(v)*u − len(u)*v), len(len(v)*u + len(u)*v))
    let len_v_mul_u = v.norm_sqr().sqrt() * u;
    let len_u_mul_v = u.norm_sqr().sqrt() * v;
    let left = (len_v_mul_u - len_u_mul_v).norm_sqr().sqrt(); // this is positive
    let right = (len_v_mul_u + len_u_mul_v).norm_sqr().sqrt(); // this is positive
    left.atan2(right) / ::std::f32::consts::PI
}

// returns biquad lowpass filter
fn get_lowpass(n: f32, q: f32) -> Box<dyn FnMut(f32) -> f32> {
    let k = (0.5 * n * ::std::f32::consts::PI).tan();
    let norm = 1.0 / (1.0 + k / q + k * k);
    let a0 = k * k * norm;
    let a1 = 2.0 * a0;
    let a2 = a0;
    let b1 = 2.0 * (k * k - 1.0) * norm;
    let b2 = (1.0 - k / q + k * k) * norm;

    let mut w1 = 0.0;
    let mut w2 = 0.0;
    // \ y[n]=b_{0}w[n]+b_{1}w[n-1]+b_{2}w[n-2],
    // where
    // w[n]=x[n]-a_{1}w[n-1]-a_{2}w[n-2].
    Box::new(move |x| {
        let w0 = x - b1 * w1 - b2 * w2;
        let y = a0 * w0 + a1 * w1 + a2 * w2;
        w2 = w1;
        w1 = w0;
        y
    })
}

// FIR analytical signal transform of length n with zero padding to be length m
// real part removes DC and nyquist, imaginary part phase shifts by 90
// should act as bandpass (remove all negative frequencies + DC & nyquist)
fn make_analytic(n: usize, m: usize) -> Vec<Complex<f32>> {
    use ::std::f32::consts::PI;
    assert_eq!(n % 2, 1, "n should be odd");
    assert!(n <= m, "n should be less than or equal to m");
    // let a = 2.0 / n as f32;
    let fft = Radix4::new(m, false);

    let mut impulse = vec![Complex::new(0.0, 0.0); m];
    let mut freqs = impulse.clone();

    let mid = (n - 1) / 2;

    impulse[mid].re = 1.0;
    let re = -1.0 / (mid - 1) as f32;
    for i in 1..mid+1 {
        if i % 2 == 0 {
            impulse[mid + i].re = re;
            impulse[mid - i].re = re;
        } else {
            let im = 2.0 / PI / i as f32;
            impulse[mid + i].im = im;
            impulse[mid - i].im = -im;
        }
        // hamming window
        let k = 0.53836 + 0.46164 * (i as f32 * PI / (mid + 1) as f32).cos();
        impulse[mid + i] = impulse[mid + i].scale(k);
        impulse[mid - i] = impulse[mid - i].scale(k);
    }
    fft.process(&mut impulse, &mut freqs);
    freqs
}

#[test]
fn test_analytic() {
    let m = 1024; // ~ 40hz
    let n = m / 4 * 3 - 1; // overlap 75%
    let freqs = make_analytic(n, m);
    // DC is below -6db
    assert!(10.0 * freqs[0].norm_sqr().log(10.0) < -6.0);
    // 40hz is above 0db
    assert!(10.0 * freqs[1].norm_sqr().log(10.0) > 0.0);
    // -40hz is below -12db
    assert!(10.0 * freqs[m-1].norm_sqr().log(10.0) < -12.0);
    // actually these magnitudes are halved bc passband is +6db
}

#[test]
fn test_lowpass() {
    let mut lp = get_lowpass(0.5, 0.71);
    println!();
    println!("{}", lp(1.0));
    for _ in 0..10 {
        println!("{}", lp(0.0));
    }
    for _ in 0..10 {
        assert!(lp(0.0).abs() < 0.5); // if it's unstable, it'll be huge
    }
}
