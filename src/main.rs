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
    let mut file = File::open(filename).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents
}

fn main() {
    display();
}

fn display() {
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

    let n = 4096;
    let ys_data: Vec<_> = (0..n).map(|_| Scalar { v: 0.0 }).collect();
    let ys = VertexBuffer::dynamic(&display, &ys_data).unwrap();
    let indices = NoIndices(PrimitiveType::LineStrip);
    let v_shader = load_from_file("src/vert.glsl");
    let h_shader = load_from_file("src/frag.glsl");
    let program = Program::from_source(&display, &v_shader, &h_shader, None).unwrap();

    let params = DrawParameters {.. Default::default() };

    let mut t: f32 = -0.5;
    let dt = 0.2;
    let k = 3.5;
    loop {
        t += dt;
        // would normalyl pass in a buffer instead of generating a new vector every time
        let next_ys: Vec<_> = (0..n)
            .map(|i: u32| (i as f32) / (n as f32))
            .map(|x| Scalar { v: ((x * std::f32::consts::PI).sin()) * ((k * x).exp() + t).sin() })
            .collect();
        ys.write(&next_ys);

        let uniforms = uniform! { n: n as f32 };
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
