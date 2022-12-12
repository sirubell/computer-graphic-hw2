use cgmath::Vector3;
use glium::uniforms::{AsUniformValue, Uniforms, UniformsStorage};

pub struct PointLight {
    pub position: Vector3<f32>,
    pub intensity: Vector3<f32>,
}

impl PointLight {
    pub fn new() -> Self {
        PointLight::new_pi(
            Vector3 {
                x: 0.8,
                y: 0.0,
                z: 0.8,
            },
            Vector3 {
                x: 0.5,
                y: 0.1,
                z: 0.1,
            },
        )
    }
    pub fn new_pi(position: Vector3<f32>, intensity: Vector3<f32>) -> Self {
        PointLight {
            position,
            intensity,
        }
    }

    pub fn add_uniforms<'a, T, R>(
        &self,
        uniforms: UniformsStorage<'a, T, R>,
    ) -> UniformsStorage<'a, [f32; 3], UniformsStorage<'a, [f32; 3], UniformsStorage<'a, T, R>>>
    where
        T: AsUniformValue,
        R: Uniforms,
    {
        uniforms
            .add("point_light_pos", self.position.into())
            .add("point_light_intensity", self.intensity.into())
    }

    pub fn shift(&mut self, x: f32, y: f32, z: f32) {
        self.position = Vector3 {
            x: self.position.x + x,
            y: self.position.y + y,
            z: self.position.z + z,
        }
    }
}

pub struct SpotLight {
    pub point_light: PointLight,
    direction: Vector3<f32>,
    cutoff_start_deg: f32,
    total_width_deg: f32,
}

impl SpotLight {
    pub fn new() -> Self {
        SpotLight {
            point_light: PointLight::new_pi(
                Vector3 {
                    x: 0.0,
                    y: 1.0,
                    z: 1.0,
                },
                Vector3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.1,
                },
            ),
            direction: Vector3 {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            cutoff_start_deg: 30.0,
            total_width_deg: 45.0,
        }
    }

    pub fn add_uniforms<'a, T, R>(
        &self,
        uniforms: UniformsStorage<'a, T, R>,
    ) -> UniformsStorage<
        'a,
        f32,
        UniformsStorage<
            'a,
            f32,
            UniformsStorage<
                'a,
                [f32; 3],
                UniformsStorage<
                    'a,
                    [f32; 3],
                    UniformsStorage<'a, [f32; 3], UniformsStorage<'a, T, R>>,
                >,
            >,
        >,
    >
    where
        T: AsUniformValue,
        R: Uniforms,
    {
        uniforms
            .add("spot_light_pos", self.point_light.position.into())
            .add("spot_light_intensity", self.point_light.intensity.into())
            .add("spot_light_dir", self.direction.into())
            .add("cutoff_start", self.cutoff_start_deg)
            .add("total_width", self.total_width_deg)
    }
}

pub struct DirectionalLight {
    direction: Vector3<f32>,
    radiance: Vector3<f32>,
    // direction = glm::vec3(1.5f, 1.5f, 1.5f);
    // radiance  = glm::vec3(1.0f, 1.0f, 1.0f);
}

impl DirectionalLight {
    pub fn new() -> DirectionalLight {
        DirectionalLight {
            direction: Vector3 {
                x: 0.0,
                y: 0.0,
                z: -1.0,
            },
            radiance: Vector3 {
                x: 0.6,
                y: 0.6,
                z: 0.6,
            },
        }
    }

    pub fn add_uniforms<'a, T, R>(
        &self,
        uniforms: UniformsStorage<'a, T, R>,
    ) -> UniformsStorage<'a, [f32; 3], UniformsStorage<'a, [f32; 3], UniformsStorage<'a, T, R>>>
    where
        T: AsUniformValue,
        R: Uniforms,
    {
        uniforms
            .add("dir_light_dir", self.direction.into())
            .add("dir_light_radiance", self.radiance.into())
    }
}
