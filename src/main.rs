use camera::Camera;

use glium::{
    self,
    glutin::event::{ElementState, VirtualKeyCode},
    uniform, DrawParameters, Program, Surface,
};

use imgui;
use imgui_glium_renderer;
use imgui_winit_support;

use tinyfiledialogs;

use light::{DirectionalLight, PointLight, SpotLight};
use mesh::TriangleMesh;
use scene_obj::{SceneLight, SceneObject};

mod camera;
mod light;
mod mesh;
mod scene_obj;

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

    let mesh = TriangleMesh::new(&display, "models/Ivysaur/Ivysaur.obj", true).unwrap();
    let mut scene_obj = SceneObject::new(mesh);

    let program = Program::from_source(
        &display,
        include_str!("shaders/phong_shading.vs"),
        include_str!("shaders/phong_shading.fs"),
        None,
    )
    .unwrap();
    let light_program = Program::from_source(
        &display,
        include_str!("shaders/point_light.vs"),
        include_str!("shaders/point_light.fs"),
        None,
    )
    .unwrap();

    let (framebuffer_width, framebuffer_height) = display.get_framebuffer_dimensions();
    let aspect_ratio = framebuffer_width as f32 / framebuffer_height as f32;

    let mut cur_rotation_y: f32 = 0.0;
    let rotate_dir_y: f32 = 1.0;
    let rotate_step: f32 = 0.5;

    let mut camera = Camera::new(aspect_ratio);
    let point_light = PointLight::new();
    let mut spot_light = SpotLight::new();
    let dir_light = DirectionalLight::new();

    let ambient_light: [f32; 3] = [0.005, 0.005, 0.005];

    let uniforms = uniform! {
        ambient_light: ambient_light,
    };

    event_loop.run(move |event, _, control_flow| {
        match event {
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
                if ui.button("change model") {
                    if let Some(file_path) = tinyfiledialogs::open_file_dialog(
                        "Choose a Model",
                        "./models",
                        Some((&["*.obj"], "obj model")),
                    ) {
                        let mesh = TriangleMesh::new(&display, &file_path, true).unwrap();
                        scene_obj = SceneObject::new(mesh);
                    }
                }

                let gl_window = display.gl_window();
                let mut frame = display.draw();
                frame.clear_color(0.44, 0.57, 0.75, 1.0);
                frame.clear_depth(1.0);
                {
                    cur_rotation_y += rotate_dir_y * rotate_step;
                    let s = cgmath::Matrix4::from_scale(1.0);
                    let r = cgmath::Matrix4::from_angle_y(cgmath::Deg(cur_rotation_y));
                    let world_matrix = s * r;

                    scene_obj.set_world_matrix(world_matrix);
                    camera.set_world_matrix(world_matrix);

                    let uniforms = scene_obj.add_uniforms(uniforms);
                    let uniforms = camera.add_uniforms(uniforms);
                    let uniforms = point_light.add_uniforms(uniforms);
                    let uniforms = spot_light.add_uniforms(uniforms);
                    let uniforms = dir_light.add_uniforms(uniforms);

                    scene_obj.draw(&display, &mut frame, &program, uniforms).unwrap();
                }
                {
                    let point_light_scene = SceneLight::new(&display, &point_light);
                    let spot_light_scene = SceneLight::new(&display, &spot_light.point_light);

                    point_light_scene
                        .draw(&mut frame, &light_program, uniforms)
                        .unwrap();
                    spot_light_scene
                        .draw(&mut frame, &light_program, uniforms)
                        .unwrap();
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
                event: glium::glutin::event::WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if input.virtual_keycode.is_some() && input.state == ElementState::Pressed {
                    let step = 0.05;
                    match input.virtual_keycode.unwrap() {
                        VirtualKeyCode::W => spot_light.point_light.shift(0.0, step, 0.0),
                        VirtualKeyCode::A => spot_light.point_light.shift(-step, 0.0, 0.0),
                        VirtualKeyCode::S => spot_light.point_light.shift(0.0, -step, 0.0),
                        VirtualKeyCode::D => spot_light.point_light.shift(step, 0.0, 0.0),
                        _ => {}
                    }
                }
            }

            glium::glutin::event::Event::WindowEvent {
                event: glium::glutin::event::WindowEvent::Resized(new_window_size),
                ..
            } => camera
                .set_aspect_ratio(new_window_size.width as f32 / new_window_size.height as f32),

            event => {
                let gl_window = display.gl_window();
                platform.handle_event(imgui_context.io_mut(), gl_window.window(), &event);
            }
        }
    });
}
