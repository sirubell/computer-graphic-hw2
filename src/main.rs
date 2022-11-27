use std::fs;

use cgmath::{self, Vector3};

use glium::{self, Display, IndexBuffer, VertexBuffer};
use glium::{Program, Surface};

use imgui;
use imgui_glium_renderer;
use imgui_winit_support;

#[derive(Copy, Clone)]
struct VertexPTN {
    position: [f32; 3],
    normal: [f32; 3],
    texcoord: [f32; 2],
}

glium::implement_vertex!(VertexPTN, position, normal, texcoord);

struct TriangleMesh {
    vertices: Vec<VertexPTN>,
    vertex_indices: Vec<u32>,

    obj_center: cgmath::Vector3<f32>,
    obj_extent: cgmath::Vector3<f32>,

    vertex_buffer: VertexBuffer<VertexPTN>,
    index_buffer: IndexBuffer<u32>,
}

impl TriangleMesh {
    fn new(
        display: &Display,
        file_path: &str,
        normalize: bool,
    ) -> Result<TriangleMesh, Box<dyn std::error::Error>> {
        let mut vertices = Vec::new();
        let mut vertex_indices = Vec::new();

        let file = fs::read_to_string(file_path)?;
        let mut positions = Vec::<cgmath::Vector3<f32>>::new();
        let mut texcoords = Vec::<cgmath::Vector2<f32>>::new();
        let mut normals = Vec::<cgmath::Vector3<f32>>::new();

        for mut line in file.lines() {
            if let Some(index) = line.find('#') {
                line = &line[0..index];
            }
            line = line.trim();

            let mut data = line.split_whitespace();
            if let Some(first_word) = data.next() {
                match first_word {
                    "v" => {
                        let x: f32 = data.next().unwrap().parse().unwrap();
                        let y: f32 = data.next().unwrap().parse().unwrap();
                        let z: f32 = data.next().unwrap().parse().unwrap();
                        positions.push(cgmath::vec3(x, y, z));
                    }
                    "vt" => {
                        let u: f32 = data.next().unwrap().parse().unwrap();
                        let v: f32 = data.next().unwrap().parse().unwrap();
                        texcoords.push(cgmath::vec2(u, v));
                    }
                    "vn" => {
                        let x: f32 = data.next().unwrap().parse().unwrap();
                        let y: f32 = data.next().unwrap().parse().unwrap();
                        let z: f32 = data.next().unwrap().parse().unwrap();
                        normals.push(cgmath::vec3(x, y, z));
                    }
                    "f" => {
                        let mut vertices_count = 0;
                        for v in data {
                            vertices_count += 1;

                            let mut indices = v.split('/');
                            let p_index = translate_index(
                                positions.len(),
                                indices.next().unwrap().parse::<i32>().unwrap(),
                            );
                            let uv_index = translate_index(
                                texcoords.len(),
                                indices.next().unwrap().parse::<i32>().unwrap(),
                            );
                            let n_index = translate_index(
                                normals.len(),
                                indices.next().unwrap().parse::<i32>().unwrap(),
                            );

                            vertices.push(VertexPTN {
                                position: positions[p_index as usize].into(),
                                normal: normals[n_index as usize].into(),
                                texcoord: texcoords[uv_index as usize].into(),
                            });

                            fn translate_index(size: usize, index: i32) -> u32 {
                                if index < 0 {
                                    (size as i32 + index).try_into().unwrap()
                                } else {
                                    (index - 1).try_into().unwrap()
                                }
                            }
                        }
                        for i in 2..vertices_count {
                            vertex_indices.push((vertices.len() - vertices_count) as u32);
                            vertex_indices.push((vertices.len() - vertices_count + i - 1) as u32);
                            vertex_indices.push((vertices.len() - vertices_count + i) as u32);
                        }
                    }
                    "mtllib" => {
                        // todo!()
                    }
                    "usemtl" | "g" => {
                        // todo!()
                    }
                    _ => {
                        // dbg!(first_word);
                    }
                }
            }
        }
        // calculate center and extent
        let mut min_extent = positions.first().unwrap().clone();
        let mut max_extent = positions.first().unwrap().clone();
        for vp in positions.iter() {
            min_extent.x = min_extent.x.min(vp.x);
            min_extent.y = min_extent.y.min(vp.y);
            min_extent.z = min_extent.z.min(vp.z);

            max_extent.x = max_extent.x.max(vp.x);
            max_extent.y = max_extent.y.max(vp.y);
            max_extent.z = max_extent.z.max(vp.z);
        }
        let mut obj_extent = max_extent - min_extent;
        let mut obj_center = (max_extent + min_extent) / 2.0;

        if normalize {
            let max_length = max_extent.x.max(max_extent.y).max(max_extent.z);
            for mut v in vertices.iter_mut() {
                v.position = ((Vector3::from(v.position) - obj_center) / max_length).into();
            }
            obj_center = Vector3::new(0.0, 0.0, 0.0);
            obj_extent /= max_length;
        }

        let vertex_buffer = VertexBuffer::new(display, &vertices)?;
        let index_buffer = IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &vertex_indices,
        )?;

        Ok(TriangleMesh {
            vertices,
            vertex_indices,
            obj_center,
            obj_extent,
            vertex_buffer,
            index_buffer,
        })
    }

    fn draw<T: Surface, U: glium::uniforms::Uniforms>(
        &self,
        frame: &mut T,
        program: &Program,
        uniforms: U,
    ) -> Result<(), glium::DrawError> {
        frame.draw(
            &self.vertex_buffer,
            &self.index_buffer,
            &program,
            &uniforms,
            &Default::default(),
        )
    }
}

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

    let mesh = TriangleMesh::new(&display, "./models/Bunny/Bunny.obj", true).unwrap();
    let program = Program::from_source(
        &display,
        include_str!("shaders/default.vs"),
        include_str!("shaders/default.fs"),
        None,
    )
    .unwrap();

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
            frame.clear_color(1.0, 1.0, 1.0, 1.0);
            {
                mesh.draw(&mut frame, &program, glium::uniforms::EmptyUniforms)
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

        event => {
            let gl_window = display.gl_window();
            platform.handle_event(imgui_context.io_mut(), gl_window.window(), &event);
        }
    });
}
