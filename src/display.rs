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
use glium::glutin::{
    WindowBuilder,
    Event,
};

use file_loader::load_from_file;
use config::{
    Config,
    Uniforms
};
use audio::{
    MultiBuffer,
};

#[derive(Copy, Clone)]
pub struct Scalar {
    pub v: f32
}
implement_vertex!(Scalar, v);

#[derive(Copy, Clone)]
pub struct Vector {
    vec: [f32; 2],
}
implement_vertex!(Vector, vec);

pub fn display(config: &Config, buffers: MultiBuffer) {
    let display = WindowBuilder::new()
        // .with_multisampling(4) // THIS IS LAGGY!
        .with_vsync()
        .build_glium().unwrap();

    let n = config.audio.fft_size;
    let mut ys_data: Vec<_> = (0..n).map(|_| Scalar { v: 0.0 }).collect();
    let ys = VertexBuffer::dynamic(&display, &ys_data).unwrap();
    let indices = NoIndices(PrimitiveType::LineStripAdjacency);
    let wave_program = Program::from_source(
        &display,
        &load_from_file("src/glsl/line.vert"),
        &load_from_file("src/glsl/line.frag"),
        Some(&load_from_file("src/glsl/line.geom"))
    ).unwrap();

    let clear_rect = [[-1.0, -1.0], [-1.0, 1.0], [1.0, -1.0], [1.0, 1.0]].into_iter()
        .map(|&v| Vector { vec: v })
        .collect::<Vec<_>>();
    let clear_rect_verts = VertexBuffer::new(&display, &clear_rect).unwrap();
    let clear_rect_indices = NoIndices(PrimitiveType::TriangleStrip);
    let clear_program = Program::from_source(
        &display,
        &load_from_file("src/glsl/clear.vert"),
        &load_from_file("src/glsl/clear.frag"),
        None
    ).unwrap();

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
        for ev in display.poll_events() {
            match ev {
                Event::Closed => return,
                _ => {}
            }
        }

        let mut target = display.draw();
        while { !buffers[index].lock().unwrap().rendered } {
            {
                let mut buffer = buffers[index].lock().unwrap();
                ys_data.copy_from_slice(&buffer.freq);
                buffer.rendered = true;
            };
            ys.write(&ys_data);
            index = (index + 1) % buffers.len();

            let window = display.get_window().unwrap();
            let (width, height) = window.get_inner_size_points().unwrap();

            let uniforms = uniform! {
                n: n / 4,
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
    }
}
