extern crate toml;
extern crate rustc_serialize;

fn main() {
    let config = load_config();
    display(config);
}

#[derive(Debug, RustcDecodable)]
struct Config {
    n: u32,
    t: f32,
    dt: f32,
    k: f32,
    a: f32,
    uniforms: Uniforms
}
#[derive(Debug, RustcDecodable)]
struct Uniforms {
    thickness: f32,
    thinning: f32,
    base_hue: f32,
    colorize: bool,
}

#[macro_use]
extern crate glium;

#[derive(Copy, Clone)]
struct Scalar {
    v: f32
}
implement_vertex!(Scalar, v);

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

fn display(config: Config) {
    let Config {
        n,
        t,
        dt,
        k,
        a,
        uniforms
    } = config;
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
    };
    use glium::index::{
        NoIndices,
        PrimitiveType
    };

    let display = WindowBuilder::new()
        .with_multisampling(4)
        .with_vsync()
        .build_glium().unwrap();

    let ys_data: Vec<_> = (0..n).map(|_| Scalar { v: 0.0 }).collect();
    let ys = VertexBuffer::dynamic(&display, &ys_data).unwrap();
    let indices = NoIndices(PrimitiveType::LineStripAdjacency);
    let v_shader = load_from_file("src/line.vert");
    let h_shader = load_from_file("src/line.frag");
    let g_shader = load_from_file("src/line.geom");
    let program = Program::from_source(&display, &v_shader, &h_shader, Some(&g_shader)).unwrap();

    let params = DrawParameters {.. Default::default() };

    let Uniforms {
        thickness,
        thinning,
        base_hue,
        colorize,
    } = uniforms;
    let mut t = t;
    loop {
        t += dt;
        // would normalyl pass in a buffer instead of generating a new vector every time
        let next_ys: Vec<_> = (0..n)
            .map(|i: u32| (i as f32) / (n as f32))
            .map(|x| Scalar { v: a * ((x * std::f32::consts::PI).sin()) * ((k * x).exp() + t).sin() })
            .collect();
        ys.write(&next_ys);

        let window = display.get_window().unwrap();
        let (width, height) = window.get_inner_size_points().unwrap();

        let uniforms = uniform! {
            n: n,
            window: [width as f32, height as f32],
            thickness: thickness,
            thinning: thinning,
            base_hue: base_hue,
            colorize: colorize,
        };
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(&ys, &indices, &program, &uniforms, &params).unwrap();
        target.finish().unwrap();
        for ev in display.poll_events() {
            match ev {
                Event::Closed => return,
                _ => {}
            }
        }
    }
}
