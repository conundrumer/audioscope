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
use rustfft::FFT;

use config::Config;
use display::Scalar;

pub type MultiBuffer = Arc<Vec<Mutex<AudioBuffer>>>;
pub type PortAudioStream = Stream<NonBlocking, Input<f32>>;

pub struct AudioBuffer {
    pub rendered: bool,
    pub time: Vec<Scalar>,
    pub freq: Vec<Scalar>,
}

const SAMPLE_RATE: f64 = 44_100.0;
const CHANNELS: i32 = 2;
const INTERLEAVED: bool = true;

pub fn init_audio(config: &Config) -> Result<(PortAudioStream, MultiBuffer), portaudio::Error> {
    let buffer_size = config.audio.buffer_size as usize;
    let fft_size = config.audio.fft_size as usize;
    let num_buffers = config.audio.num_buffers;

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
            time: vec![Scalar {v: 0.0}; buffer_size],
            freq: vec![Scalar {v: 0.0}; fft_size], // magnitude only for now
        }));
    }
    let buffers = Arc::new(buffers);

    let (receiver, callback) = {
        let mut buffer_index = 0;
        let (sender, receiver) = mpsc::channel();
        let print_drop = config.debug.print_drop;
        let buffers = buffers.clone();
        let mut time_buffer = vec![Scalar {v: 0.0}; buffer_size];
        let mut freq_buffer = vec![Scalar {v: 0.0}; fft_size];

        let mut time_index = 0;
        let mut time_ring_buffer = vec![Complex::new(0.0, 0.0); 2 * fft_size];
        let mut complex_freq_buffer = vec![Complex::new(0.0f32, 0.0); fft_size];

        let mut fft = FFT::new(fft_size, false);
        let buffer_size = buffer_size as f32;

        (receiver, move |InputStreamCallbackArgs { buffer: data, .. }| {
            {
                let (left, right) = time_ring_buffer.split_at_mut(fft_size);
                for (((x, y), t0), t1) in data.chunks(CHANNELS as usize)
                    .zip(time_buffer.iter_mut())
                    .zip(left.iter_mut().skip(time_index))
                    .zip(right.iter_mut().skip(time_index))
                {
                    let mono = (x[0] + x[1]) / 2.0;
                    y.v = mono;
                    let mono = Complex::new(mono, 0.0);
                    *t0 = mono;
                    *t1 = mono;
                }
            }
            fft.process(&time_ring_buffer[time_index..time_index + fft_size], &mut complex_freq_buffer[..]);
            time_index += (time_index + buffer_size as usize) % fft_size;

            for (x, y) in complex_freq_buffer.iter().zip(freq_buffer.iter_mut()) {
                y.v = x.norm_sqr().sqrt() / buffer_size;
            }

            let dropped = {
                let mut buffer = buffers[buffer_index].lock().unwrap();
                let rendered = buffer.rendered;
                buffer.time.copy_from_slice(&time_buffer);
                buffer.freq.copy_from_slice(&freq_buffer);
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

// FIR analytical signal transform of length n with zero padding to be length m
// real part removes DC and nyquist, imaginary part phase shifts by 90
// should act as bandpass (remove all negative frequencies + DC & nyquist)
fn make_analytic(n: usize, m: usize) -> Vec<Complex<f32>> {
    use ::std::f32::consts::PI;
    assert_eq!(n % 2, 1, "n should be odd");
    assert!(n <= m, "n should be less than or equal to m");
    // let a = 2.0 / n as f32;
    let mut fft = FFT::new(m, false);

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
    fft.process(&impulse, &mut freqs);
    freqs
}

#[test]
fn analytic() {
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
