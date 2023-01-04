use std::{collections::HashMap, fs, path::Path};

use cgmath::{vec3, Vector2, Vector3};
use glium::{
    uniforms::{AsUniformValue, MagnifySamplerFilter, Sampler, Uniforms, UniformsStorage},
    Display, DrawParameters, IndexBuffer, Program, Surface, Texture2d, VertexBuffer,
};
use image::io::Reader;

#[derive(Copy, Clone, Debug)]
struct VertexPTN {
    position: [f32; 3],
    texcoord: [f32; 2],
    normal: [f32; 3],
}

glium::implement_vertex!(VertexPTN, position, normal, texcoord);

#[derive(Clone, Debug)]
struct Material {
    ns: f32,
    ka: Vector3<f32>,
    kd: Option<Vector3<f32>>,
    ks: Vector3<f32>,
    mapkd: Vec<Vec<(u8, u8, u8)>>,
}

struct SubMesh {
    name: Option<String>,
    vertex_buffer: VertexBuffer<VertexPTN>,
    index_buffer: IndexBuffer<u32>,
    material: Material,
}

impl SubMesh {
    fn new(
        display: &Display,
        vertices: &[VertexPTN],
        vertex_indices: &[u32],
        material: Material,
        group_name: Option<&str>,
    ) -> SubMesh {
        let name = match group_name {
            Some(name) => Some(String::from(name)),
            None => None,
        };

        let vertex_buffer = VertexBuffer::new(display, &vertices).unwrap();
        let index_buffer = IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &vertex_indices,
        )
        .unwrap();

        SubMesh {
            name,
            vertex_buffer,
            index_buffer,
            material,
        }
    }

    fn add_uniforms<'a, T, R>(
        &self,
        uniforms: UniformsStorage<'a, T, R>,
    ) -> UniformsStorage<
        'a,
        [f32; 3],
        UniformsStorage<
            'a,
            [f32; 3],
            UniformsStorage<'a, [f32; 3], UniformsStorage<'a, f32, UniformsStorage<'a, T, R>>>,
        >,
    >
    where
        T: AsUniformValue,
        R: Uniforms,
    {
        let ns = self.material.ns;
        let ka = <Vector3<f32> as Into<[f32; 3]>>::into(self.material.ka);
        let ks = <Vector3<f32> as Into<[f32; 3]>>::into(self.material.ks);
        let kd = match self.material.kd {
            Some(v) => <Vector3<f32> as Into<[f32; 3]>>::into(v),
            None => [0.0, 0.0, 0.0],
        };

        uniforms
            .add("ns", ns)
            .add("ka", ka)
            .add("kd", kd)
            .add("ks", ks)
    }

    fn draw<S, T, R>(
        &self,
        display: &Display,
        frame: &mut S,
        program: &Program,
        uniforms: UniformsStorage<T, R>,
    ) -> Result<(), glium::DrawError>
    where
        S: Surface,
        T: AsUniformValue,
        R: Uniforms,
    {
        use std::time::Instant;

        let now = Instant::now();
        let mapkd = if !self.material.mapkd.is_empty() {
            Texture2d::new(display, self.material.mapkd.clone()).unwrap()
        } else {
            Texture2d::empty(display, 0, 0).unwrap()
        };
        let mapkd = mapkd
            .sampled()
            .magnify_filter(MagnifySamplerFilter::Linear);
        let elasped = now.elapsed();
        dbg!(elasped);

        let uniforms = uniforms.add("mapkd", mapkd);
        dbg!(mapkd);

        frame.draw(
            &self.vertex_buffer,
            &self.index_buffer,
            &program,
            &self.add_uniforms(uniforms),
            &DrawParameters {
                depth: glium::Depth {
                    test: glium::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
    }
}

pub struct TriangleMesh {
    obj_center: Vector3<f32>,
    obj_extent: Vector3<f32>,
    submeshes: Vec<SubMesh>,
}

impl TriangleMesh {
    pub fn new(
        display: &Display,
        file_path: &str,
        normalize: bool,
    ) -> Result<TriangleMesh, Box<dyn std::error::Error>> {
        let mut vertices = Vec::new();
        let mut vertex_indices = Vec::new();
        let mut prev_index: usize = 0;
        let mut cur_index: usize = 0;

        let mut materials = HashMap::<String, Material>::new();
        let mut submesh_attr: Vec<(&str, Option<&str>, [usize; 2])> = Vec::new();

        let mut positions = Vec::<Vector3<f32>>::new();
        let mut texcoords = Vec::<Vector2<f32>>::new();
        let mut normals = Vec::<Vector3<f32>>::new();

        let mut group_name: Option<&str> = None;
        let mut mtl_name: Option<&str> = None;

        let parent_path = std::path::Path::new(file_path).parent().unwrap();
        let file = fs::read_to_string(file_path)?;

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

                        cur_index += (vertices_count - 2) * 3;
                    }
                    "mtllib" => {
                        let mtl_file_name = data.next().unwrap();
                        Self::load_mtl(
                            parent_path.join(mtl_file_name).to_str().unwrap(),
                            &mut materials,
                        )?;
                    }
                    "g" => {
                        group_name = Some(data.next().unwrap());
                    }
                    "usemtl" => {
                        // todo!()
                        if mtl_name.is_some() {
                            submesh_attr.push((
                                mtl_name.unwrap(),
                                group_name,
                                [prev_index, cur_index],
                            ));
                        }

                        mtl_name = Some(data.next().unwrap());
                        prev_index = cur_index;
                    }
                    _ => {
                        // dbg!(first_word);
                    }
                }
            }
        }

        submesh_attr.push((mtl_name.unwrap(), group_name, [prev_index, cur_index]));

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

        let mut submeshes: Vec<SubMesh> = Vec::new();
        for attr in &submesh_attr {
            submeshes.push(SubMesh::new(
                display,
                &vertices[..],
                &vertex_indices[attr.2[0]..attr.2[1]],
                materials[attr.0].clone(),
                attr.1,
            ));
        }

        Ok(TriangleMesh {
            obj_center,
            obj_extent,
            submeshes,
        })
    }

    fn load_mtl(
        file_path: &str,
        materials: &mut HashMap<String, Material>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = fs::read_to_string(file_path)?;

        let mut mtl_name: Option<&str> = None;
        let mut ns: Option<f32> = None;
        let mut ka: Option<Vector3<f32>> = None;
        let mut kd: Option<Vector3<f32>> = None;
        let mut ks: Option<Vector3<f32>> = None;
        let mut mapkd: Vec<Vec<(u8, u8, u8)>> = Vec::new();

        for mut line in file.lines() {
            if let Some(index) = line.find('#') {
                line = &line[0..index];
            }
            line = line.trim();

            let mut data = line.split_whitespace();

            if let Some(first_word) = data.next() {
                match first_word {
                    "newmtl" => {
                        if mtl_name.is_some() {
                            materials.insert(
                                String::from(mtl_name.unwrap()),
                                Material {
                                    ns: ns.unwrap(),
                                    ka: ks.unwrap(),
                                    kd,
                                    ks: ks.unwrap(),
                                    mapkd,
                                },
                            );
                            kd = None;
                            mapkd = Vec::new();
                        }

                        mtl_name = Some(data.next().unwrap());
                    }
                    "Ns" => {
                        let x: f32 = data.next().unwrap().parse().unwrap();
                        ns = Some(x);
                    }
                    "Ka" => {
                        let x: f32 = data.next().unwrap().parse().unwrap();
                        let y: f32 = data.next().unwrap().parse().unwrap();
                        let z: f32 = data.next().unwrap().parse().unwrap();
                        ka = Some(vec3(x, y, z))
                    }
                    "Kd" => {
                        let x: f32 = data.next().unwrap().parse().unwrap();
                        let y: f32 = data.next().unwrap().parse().unwrap();
                        let z: f32 = data.next().unwrap().parse().unwrap();
                        kd = Some(vec3(x, y, z))
                    }
                    "Ks" => {
                        let x: f32 = data.next().unwrap().parse().unwrap();
                        let y: f32 = data.next().unwrap().parse().unwrap();
                        let z: f32 = data.next().unwrap().parse().unwrap();
                        ks = Some(vec3(x, y, z))
                    }
                    "map_Kd" => {
                        let texture_path = data.next().unwrap();
                        let mut parent_path = Path::new(file_path).to_path_buf();
                        parent_path.pop();
                        let texture_path = parent_path.join(texture_path);
                        dbg!(&texture_path);
                        let texture_image = Reader::open(texture_path)?.decode()?;
                        let texture_image = texture_image.into_rgb8();

                        let mut buffer: Vec<Vec<(u8, u8, u8)>> = Vec::new();
                        for x in 0..texture_image.width() {
                            buffer.push(Vec::new());
                            for y in 0..texture_image.height() {
                                let pixel = texture_image.get_pixel(x, y).clone();
                                buffer
                                    .last_mut()
                                    .unwrap()
                                    .push((pixel.0[0], pixel.0[1], pixel.0[2]));
                            }
                        }

                        mapkd = buffer;
                    }
                    _ => {
                        // don't care
                        // unreachable!()
                    }
                }
            }
        }

        materials.insert(
            String::from(mtl_name.unwrap()),
            Material {
                ns: ns.unwrap(),
                ka: ka.unwrap(),
                kd,
                ks: ks.unwrap(),
                mapkd,
            },
        );

        Ok(())
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
        for submesh in &self.submeshes {
            submesh.draw(display, frame, program, uniforms.clone())?;
        }
        Ok(())
    }
}
