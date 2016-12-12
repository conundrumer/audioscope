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
    let n = config.audio.buffer_size as usize;

    let pa = PortAudio::new()?;

    let def_input = pa.default_input_device()?;
    let input_info = pa.device_info(def_input)?;
    println!("Default input device name: {}", input_info.name);

    let latency = input_info.default_low_input_latency;
    let input_params = StreamParameters::<f32>::new(def_input, CHANNELS, INTERLEAVED, latency);

    pa.is_input_format_supported(input_params, SAMPLE_RATE)?;
    let settings = InputStreamSettings::new(input_params, SAMPLE_RATE, n as u32);

    let mut buffers = Vec::with_capacity(config.audio.num_buffers);
    let empty_buffer = vec![Scalar {v: 0.0}; n];
    for _ in 0..config.audio.num_buffers {
        buffers.push(Mutex::new(AudioBuffer {
            rendered: true,
            time: empty_buffer.clone(),
            freq: empty_buffer.clone(), // magnitude only for now
        }));
    }
    let buffers = Arc::new(buffers);

    let (receiver, callback) = {
        let mut index = 0;
        let (sender, receiver) = mpsc::channel();
        let print_drop = config.debug.print_drop;
        let buffers = buffers.clone();
        let mut time_buffer = empty_buffer.clone();
        let mut freq_buffer = empty_buffer.clone();
        let mut complex_time_buffer = vec![Complex::new(0.0, 0.0); n];
        let mut complex_freq_buffer = vec![Complex::new(0.0, 0.0); n];

        let n = n as f32;
        let mut fft = FFT::new(freq_buffer.len(), false);

        (receiver, move |InputStreamCallbackArgs { buffer: data, .. }| {
            for ((x, y), z) in data.chunks(CHANNELS as usize).zip(time_buffer.iter_mut()).zip(complex_time_buffer.iter_mut()) {
                let mono = (x[0] + x[1]) / 2.0;
                y.v = mono;
                *z = Complex::new(mono, 0.0);
            }
            fft.process(&complex_time_buffer[..], &mut complex_freq_buffer[..]);
            for (x, y) in complex_freq_buffer.iter().zip(freq_buffer.iter_mut()) {
                y.v = x.norm_sqr().sqrt() / n;
            }

            let dropped = {
                let mut buffer = buffers[index].lock().unwrap();
                let rendered = buffer.rendered;
                buffer.time.copy_from_slice(&time_buffer);
                buffer.freq.copy_from_slice(&freq_buffer);
                buffer.rendered = false;
                !rendered
            };
            index = (index + 1) % buffers.len();
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
