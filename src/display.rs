use glium::{
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
use glium::glutin::window::{
    WindowBuilder,
};
use glium::glutin::event::{
    Event,
    WindowEvent,
    StartCause,
};
use glium::glutin::event_loop::ControlFlow;

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
pub struct Vec2 {
    pub vec: [f32; 2],
}
implement_vertex!(Vec2, vec);

#[derive(Copy, Clone)]
pub struct Vec4 {
    pub vec: [f32; 4],
}
implement_vertex!(Vec4, vec);


pub fn display(config: &Config, buffers: MultiBuffer) {
    let el = glium::glutin::event_loop::EventLoop::new();

    let wb = WindowBuilder::new();

    let cb = glium::glutin::ContextBuilder::new()
        .with_vsync(true);

    let display = glium::Display::new(wb, cb, &el).unwrap();

    let n = config.audio.buffer_size + 3;
    let mut ys_data: Vec<_> = (0..n).map(|_| Vec4 { vec: [0.0, 0.0, 0.0, 0.0] }).collect();
    let ys = VertexBuffer::dynamic(&display, &ys_data).unwrap();
    let indices = NoIndices(PrimitiveType::LineStripAdjacency);
    let wave_program = Program::from_source(
        &display,
        &load_from_file("src/glsl/line.vert"),
        &load_from_file("src/glsl/line.frag"),
        Some(&load_from_file("src/glsl/line.geom"))
    ).unwrap();

    let clear_rect = [[-1.0, -1.0], [-1.0, 1.0], [1.0, -1.0], [1.0, 1.0]].iter()
        .map(|&v| Vec2 { vec: v })
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
        min_thickness,
        thinning,
        base_hue,
        colorize,
        desaturation,
    } = config.uniforms;

    let frame_wait_time = match config.max_fps {
        Some(fps) => std::time::Duration::new(0, 1_000_000_000 / fps),
        None => std::time::Duration::from_nanos(0),
    };

    el.run(move |event, _, control_flow| {
        let mut index = 0;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    return;
                },
                _ => return,
            },
            Event::NewEvents(cause) => match cause {
                StartCause::ResumeTimeReached { .. } => (),
                StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }

        let next_frame_time = std::time::Instant::now() + frame_wait_time;
        *control_flow = ControlFlow::WaitUntil(next_frame_time);

        let mut target = display.draw();
        while !buffers[index].lock().unwrap().rendered {
            {
                let mut buffer = buffers[index].lock().unwrap();
                ys_data.copy_from_slice(&buffer.analytic);
                buffer.rendered = true;
            };
            ys.write(&ys_data);
            index = (index + 1) % buffers.len();

            let window_size = display.gl_window().window().inner_size();

            let uniforms = uniform! {
                n: n,
                decay: decay,
                window: [window_size.width as f32, window_size.height as f32],
                thickness: thickness,
                min_thickness: min_thickness,
                thinning: thinning,
                base_hue: base_hue,
                colorize: colorize,
                desaturation: desaturation,
            };
            target.draw(&clear_rect_verts, &clear_rect_indices, &clear_program, &uniforms, &params).unwrap();
            target.draw(&ys, &indices, &wave_program, &uniforms, &params).unwrap();
        }

        target.finish().unwrap();
    });
}
