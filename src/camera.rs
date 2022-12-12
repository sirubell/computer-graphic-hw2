use cgmath::{Matrix4, Point3, Vector3};
use glium::uniforms::{AsUniformValue, Uniforms, UniformsStorage};

pub struct Camera {
    camera_pos: Point3<f32>,
    camera_dir: Vector3<f32>,
    camera_up: Vector3<f32>,
    fovy: f32,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,
    world_matrix: Matrix4<f32>,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Camera {
        let camera_pos = Point3::<f32>::new(0.0, 1.0, 5.0);
        let camera_dir = Point3::<f32>::new(0.0, 0.0, 0.0) - camera_pos;
        let camera_up = Vector3::<f32>::new(0.0, 1.0, 0.0);

        let fovy: f32 = 30.0;
        let z_near: f32 = 0.1;
        let z_far: f32 = 1000.0;

        Camera {
            camera_pos,
            camera_dir,
            camera_up,
            fovy,
            aspect_ratio,
            z_near,
            z_far,
            world_matrix: Matrix4::<f32>::from_scale(1.0),
        }
    }

    pub fn add_uniforms<'a, T, R>(
        &self,
        uniforms: UniformsStorage<'a, T, R>,
    ) -> UniformsStorage<'a, [[f32; 4]; 4], UniformsStorage<'a, T, R>>
    where
        T: AsUniformValue,
        R: Uniforms,
    {
        let mvp = self.perspective_matrix() * self.view_matrix() * self.world_matrix;

        uniforms.add("mvp", mvp.into())
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(self.camera_pos, self.camera_dir, self.camera_up)
    }

    pub fn perspective_matrix(&self) -> Matrix4<f32> {
        cgmath::perspective(
            cgmath::Deg(self.fovy),
            self.aspect_ratio,
            self.z_near,
            self.z_far,
        )
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }

    pub fn set_world_matrix(&mut self, world_matrix: Matrix4<f32>) {
        self.world_matrix = world_matrix;
    }
}
