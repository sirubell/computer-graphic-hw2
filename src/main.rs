use cgmath::{self, Matrix4, Point3, Vector3};

use glium::{self, uniform, Program, Surface};

use imgui;
use imgui_glium_renderer;
use imgui_winit_support;

use mesh::TriangleMesh;

mod mesh;

fn main() {
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let window_builder = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(1024.0, 768.0))
        .with_title("Hello World");
    let context_builder = glium::glutin::ContextBuilder::new();

    let display = glium::Display::new(window_builder, context_builder, &event_loop).unwrap();
    let mut imgui_context = imgui::Context::create();
    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);
    {
        let gl_window = display.gl_window();
        let window = gl_window.window();

        platform.attach_window(
            imgui_context.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );
    }
    let mut imgui_renderer =
        imgui_glium_renderer::Renderer::init(&mut imgui_context, &display).unwrap();

    let mut last_frame = std::time::Instant::now();

    let mesh = TriangleMesh::new(&display, "models/Koffing/Koffing.obj", true).unwrap();
    let program = Program::from_source(
        &display,
        include_str!("shaders/fixed_color.vs"),
        include_str!("shaders/fixed_color.fs"),
        None,
    )
    .unwrap();

    let camera_pos = Point3::<f32>::new(0.0, 0.5, 2.0);
    let camera_target = Point3::<f32>::new(0.0, 0.0, 0.0);
    let camera_up = Vector3::<f32>::new(0.0, 1.0, 0.0);
    let v = Matrix4::look_at_rh(camera_pos, camera_target, camera_up);
    let fov: f32 = 40.0;
    let (framebuffer_width, framebuffer_height) = display.get_framebuffer_dimensions();
    let aspect_ratio = framebuffer_width as f32 / framebuffer_height as f32;
    let z_near: f32 = 0.1;
    let z_far: f32 = 100.0;
    let p = cgmath::perspective(cgmath::Deg(fov), aspect_ratio, z_near, z_far);

    let m = Matrix4::from_scale(1.0);
    // Apply CPU transformation.
    let mvp = p * v * m;

    let fixed_color = Vector3::new(1.0, 0.5, 0.2);

    let uniforms = uniform! {
        mvp: <Matrix4<f32> as Into<[[f32; 4]; 4]>>::into(mvp),
        fixed_color: <Vector3<f32> as Into<[f32; 3]>>::into(fixed_color),
    };

    event_loop.run(move |event, _, control_flow| match event {
        glium::glutin::event::Event::NewEvents(_) => {
            imgui_context
                .io_mut()
                .update_delta_time(last_frame.elapsed());
            last_frame = std::time::Instant::now();
        }

        glium::glutin::event::Event::MainEventsCleared => {
            let gl_window = display.gl_window();
            platform
                .prepare_frame(imgui_context.io_mut(), gl_window.window())
                .expect("Failed to prepare frame");
            gl_window.window().request_redraw();
        }

        glium::glutin::event::Event::RedrawRequested(_) => {
            let ui = imgui_context.frame();

            // Added this line to try to render some text
            ui.text("test");

            let gl_window = display.gl_window();
            let mut frame = display.draw();
            frame.clear_color(0.44, 0.57, 0.75, 1.0);
            {
                mesh.draw(&mut frame, &program, &uniforms).unwrap();
            }
            platform.prepare_render(&ui, gl_window.window());
            let draw_data = ui.render();
            imgui_renderer
                .render(&mut frame, draw_data)
                .expect("UI rendering failed");
            frame.finish().expect("Failed to swap buffers");
        }

        glium::glutin::event::Event::WindowEvent {
            event: glium::glutin::event::WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
        }

        glium::glutin::event::Event::WindowEvent {
            event: glium::glutin::event::WindowEvent::Resized(new_window_size),
            ..
        } => {
            todo!();
        }

        event => {
            let gl_window = display.gl_window();
            platform.handle_event(imgui_context.io_mut(), gl_window.window(), &event);
        }
    });
}
