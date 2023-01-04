use cgmath::{Matrix4, SquareMatrix};
use glium::{
    uniforms::{AsUniformValue, Uniforms, UniformsStorage},
    Display, DrawParameters, IndexBuffer, Program, Surface, VertexBuffer,
};

use crate::{light::PointLight, TriangleMesh};

#[derive(Copy, Clone, Debug)]
struct VertexP {
    position: [f32; 3],
}

glium::implement_vertex!(VertexP, position);

pub struct SceneObject {
    mesh: TriangleMesh,
    world_matrix: Matrix4<f32>,
}

impl SceneObject {
    // let mesh = TriangleMesh::new(&display, "models/Forklift/Forklift.obj", true).unwrap();
    pub fn new(mesh: TriangleMesh) -> Self {
        Self {
            mesh,
            world_matrix: Matrix4::from_scale(1.0),
        }
    }

    pub fn set_world_matrix(&mut self, matrix: Matrix4<f32>) {
        self.world_matrix = matrix;
    }

    pub fn add_uniforms<'a, T, R>(
        &self,
        uniforms: UniformsStorage<'a, T, R>,
    ) -> UniformsStorage<
        'a,
        [[f32; 4]; 4],
        UniformsStorage<'a, [[f32; 4]; 4], UniformsStorage<'a, T, R>>,
    >
    where
        T: AsUniformValue,
        R: Uniforms,
    {
        let mut normal_matrix = self.world_matrix.invert().unwrap();
        normal_matrix.transpose_self();

        uniforms
            .add("world_matrix", self.world_matrix.into())
            .add("normal_matrix", normal_matrix.to_owned().into())
    }

    pub fn draw<S, T, R>(
        &self,
        display: &Display,
        frame: &mut S,
        program: &Program,
        uniforms: UniformsStorage<T, R>,
    ) -> Result<(), glium::DrawError>
    where
        S: Surface,
        T: AsUniformValue + Clone,
        R: Uniforms + Clone,
    {
        self.mesh.draw(display, frame, program, uniforms)?;
        Ok(())
    }
}

pub struct SceneLight<'a> {
    point_light: &'a PointLight,
    vertex_buffer: VertexBuffer<VertexP>,
    index_buffer: IndexBuffer<u32>,
}

impl<'a> SceneLight<'a> {
    pub fn new(display: &'_ Display, point_light: &'a PointLight) -> SceneLight<'a> {
        let p = VertexP {
            position: point_light.position.into(),
        };
        let vertex_buffer = VertexBuffer::new(display, &[p]).unwrap();
        let index_buffer =
            IndexBuffer::new(display, glium::index::PrimitiveType::Points, &[0]).unwrap();

        SceneLight::<'a> {
            point_light,
            vertex_buffer,
            index_buffer,
        }
    }

    fn add_uniforms<T, R>(
        &self,
        uniforms: UniformsStorage<'a, T, R>,
    ) -> UniformsStorage<'a, [f32; 3], UniformsStorage<'a, T, R>>
    where
        T: AsUniformValue,
        R: Uniforms,
    {
        uniforms.add("light_intensity", self.point_light.intensity.into())
    }

    pub fn draw<S, T, R>(
        &self,
        frame: &mut S,
        program: &Program,
        uniforms: UniformsStorage<T, R>,
    ) -> Result<(), glium::DrawError>
    where
        S: Surface,
        T: AsUniformValue,
        R: Uniforms,
    {
        frame.draw(
            &self.vertex_buffer,
            &self.index_buffer,
            &program,
            &self.add_uniforms(uniforms),
            &DrawParameters {
                point_size: Some(16.0),
                ..Default::default()
            },
        )
    }
}
