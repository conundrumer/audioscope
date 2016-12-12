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

use config::Config;
use display::Scalar;

pub type MultiBuffer = Arc<Vec<Mutex<AudioBuffer>>>;
pub type PortAudioStream = Stream<NonBlocking, Input<f32>>;

pub struct AudioBuffer {
    pub rendered: bool,
    pub data: Vec<Scalar>,
}

const SAMPLE_RATE: f64 = 44_100.0;
const CHANNELS: i32 = 2;
const INTERLEAVED: bool = true;

pub fn init_audio(config: &Config) -> Result<(PortAudioStream, MultiBuffer), portaudio::Error> {
    let pa = PortAudio::new()?;

    let def_input = pa.default_input_device()?;
    let input_info = pa.device_info(def_input)?;
    println!("Default input device name: {}", input_info.name);

    let latency = input_info.default_low_input_latency;
    let input_params = StreamParameters::<f32>::new(def_input, CHANNELS, INTERLEAVED, latency);

    pa.is_input_format_supported(input_params, SAMPLE_RATE)?;
    let settings = InputStreamSettings::new(input_params, SAMPLE_RATE, config.n);

    let mut buffers = Vec::with_capacity(config.max_buffers);
    for _ in 0..config.max_buffers {
        buffers.push(Mutex::new(AudioBuffer {
            rendered: true,
            data: vec![Scalar {v: 0.0}; config.n as usize]
        }));
    }
    let buffers = Arc::new(buffers);

    let (receiver, callback) = {
        let mut index = 0;
        let (sender, receiver) = mpsc::channel();
        let print_drop = config.debug.print_drop;
        let buffers = buffers.clone();
        let mut temp_buffer = vec![Scalar {v: 0.0}; config.n as usize];

        (receiver, move |InputStreamCallbackArgs { buffer: data, .. }| {
            for (y, x) in temp_buffer.iter_mut().zip(data.chunks(CHANNELS as usize)) {
                y.v = (x[0] + x[1]) / 2.0;
            }
            let dropped = {
                let mut buffer = buffers[index].lock().unwrap();
                let rendered = buffer.rendered;
                buffer.data.copy_from_slice(&temp_buffer);
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
