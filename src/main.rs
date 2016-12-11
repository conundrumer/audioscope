extern crate toml;
extern crate rustc_serialize;

#[macro_use]
extern crate glium;

extern crate portaudio;

type MultiBuffer = Vec<Mutex<AudioBuffer>>;

struct AudioBuffer {
    rendered: bool,
    data: Vec<f32>,
}

fn main() {
    let config = load_config();
    let (mut stream, buffer) = init_audio(&config).unwrap();
    stream.start().unwrap();
    display(&config, buffer);
    stream.stop().unwrap();
}

use std::sync::mpsc;
use std::sync::{Arc, Mutex};

type PortAudioStream = portaudio::Stream<portaudio::NonBlocking, portaudio::Input<f32>>;
const SAMPLE_RATE: f64 = 44_100.0;
const CHANNELS: i32 = 2;
const INTERLEAVED: bool = true;
fn init_audio(config: &Config) -> Result<(PortAudioStream, Arc<MultiBuffer>), portaudio::Error> {
    use portaudio::{
        PortAudio,
        StreamParameters,
        InputStreamSettings,
        InputStreamCallbackArgs,
        Continue
    };
    let pa: PortAudio = PortAudio::new()?;

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
            data: vec![0.0; CHANNELS as usize * config.n as usize]
        }));
    }
    let buffers = Arc::new(buffers);

    let (receiver, callback) = {
        let mut index = 0;
        let (sender, receiver) = mpsc::channel();
        let print_drop = config.debug.print_drop;
        let buffers = buffers.clone();

        (receiver, move |InputStreamCallbackArgs { buffer: data, .. }| {
            let dropped = {
                let mut buffer = buffers[index].lock().unwrap();
                let rendered = buffer.rendered;
                buffer.rendered = false;
                buffer.data.copy_from_slice(data);
                !rendered
            };
            index = (index + 1) % buffers.len();
            if dropped && print_drop {
                sender.send(()).ok();
            }
            Continue
        })
    };
    std::thread::spawn(move || {
        while let Ok(_) = receiver.recv() {
            print!("!");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
    });
    let stream = pa.open_non_blocking_stream(settings, callback)?;

    Ok((stream, buffers))
}

#[derive(Debug, RustcDecodable)]
struct Config {
    n: u32,
    max_buffers: usize,
    uniforms: Uniforms,
    debug: DebugConfig
}
#[derive(Debug, RustcDecodable)]
struct Uniforms {
    decay: f32,
    thickness: f32,
    thinning: f32,
    base_hue: f32,
    colorize: bool,
}
#[derive(Debug, RustcDecodable)]
struct DebugConfig {
    print_drop: bool
}

#[derive(Copy, Clone)]
struct Scalar {
    v: f32
}
implement_vertex!(Scalar, v);

#[derive(Copy, Clone)]
struct Vector {
    vec: [f32; 2],
}
implement_vertex!(Vector, vec);

fn load_from_file(filename: &str) -> String {
    use std::io::prelude::*;
    use std::fs::File;
    let mut file = File::open(filename).unwrap_or_else(|e| panic!("couldn't open file {}: {}", filename, e));
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap_or_else(|e| panic!("couldn't read file {}: {}", filename, e));
    contents
}

fn load_config() -> Config {
    use std::env;
    use rustc_serialize::Decodable;
    use toml::{Decoder, Value, Parser};

    let config_filename = env::args().nth(1)
        .unwrap_or_else(|| panic!("put in a config file"));
    let config_file = load_from_file(&config_filename);
    let mut parser = Parser::new(&config_file);
    let config = match parser.parse() {
        Some(config) => config,
        None => {
            for err in &parser.errors {
                println!("{}", err);
            }
            panic!("invalid config file");
        }
    };
    Config::decode(&mut Decoder::new(Value::Table(config)))
        .unwrap_or_else(|e| panic!("invalid config file: {}", e))
}

fn display(config: &Config, buffers: Arc<MultiBuffer>) {
    use glium::glutin::{
        WindowBuilder,
        Event,
    };
    use glium::{
        DisplayBuild,
        Surface,
        VertexBuffer,
        Program,
        DrawParameters,
        Blend,
    };
    use glium::index::{
        NoIndices,
        PrimitiveType
    };

    let display = WindowBuilder::new()
        // .with_multisampling(4) // THIS IS LAGGY!
        .with_vsync()
        .build_glium().unwrap();

    let mut ys_data: Vec<_> = (0..config.n).map(|_| Scalar { v: 0.0 }).collect();
    let ys = VertexBuffer::dynamic(&display, &ys_data).unwrap();
    let indices = NoIndices(PrimitiveType::LineStripAdjacency);
    let v_shader = load_from_file("src/line.vert");
    let h_shader = load_from_file("src/line.frag");
    let g_shader = load_from_file("src/line.geom");
    let wave_program = Program::from_source(&display, &v_shader, &h_shader, Some(&g_shader)).unwrap();

    let clear_rect = [[-1.0, -1.0], [-1.0, 1.0], [1.0, -1.0], [1.0, 1.0]].into_iter()
        .map(|&v| Vector { vec: v })
        .collect::<Vec<_>>();
    let clear_rect_verts = VertexBuffer::new(&display, &clear_rect).unwrap();
    let clear_rect_indices = NoIndices(PrimitiveType::TriangleStrip);
    let clear_program = Program::from_source(&display, &load_from_file("src/clear.vert"), &load_from_file("src/clear.frag"), None).unwrap();

    let params = DrawParameters {
        blend: Blend::alpha_blending(),
        .. Default::default()
    };

    let Uniforms {
        decay,
        thickness,
        thinning,
        base_hue,
        colorize,
    } = config.uniforms;

    let mut index = 0;
    loop {
        let mut target = display.draw();
        while { !buffers[index].lock().unwrap().rendered } {
            {
                let mut buffer = buffers[index].lock().unwrap();
                for (y, x) in ys_data.iter_mut().zip(buffer.data.chunks(CHANNELS as usize)) {
                    y.v = (x[0] + x[1]) / 2.0;
                }
                buffer.rendered = true;
            };
            ys.write(&ys_data);
            index = (index + 1) % buffers.len();

            let window = display.get_window().unwrap();
            let (width, height) = window.get_inner_size_points().unwrap();

            let uniforms = uniform! {
                n: config.n,
                decay: decay,
                window: [width as f32, height as f32],
                thickness: thickness,
                thinning: thinning,
                base_hue: base_hue,
                colorize: colorize,
            };
            target.draw(&clear_rect_verts, &clear_rect_indices, &clear_program, &uniforms, &params).unwrap();
            target.draw(&ys, &indices, &wave_program, &uniforms, &params).unwrap();
        }

        target.finish().unwrap();
        for ev in display.poll_events() {
            match ev {
                Event::Closed => return,
                _ => {}
            }
        }
    }
}
