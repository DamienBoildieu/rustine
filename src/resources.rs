// Go back to https://sotrh.github.io/learn-wgpu/beginner/tutorial9-models/#accessing-files-from-wasm if adding wasm support is needed
// TODO: Add project config system with asset folder field

use std::io::{BufReader, Cursor};

use crate::graphics::{Material, Model, ModelMesh, ModelVertex, Texture};

use futures_lite::io::{BufReader as FuturesBufReader, Cursor as FuturesCursor};
use wgpu::util::DeviceExt;

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    let txt = {
        let path = std::path::Path::new("assets").join(file_name);
        std::fs::read_to_string(path)?
    };

    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let data = {
        let path = std::path::Path::new("assets").join(file_name);
        std::fs::read(path)?
    };

    Ok(data)
}

pub async fn load_texture(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<Texture> {
    let data = load_binary(file_name).await?;
    Texture::from_bytes(device, queue, &data, file_name)
}


/// Load a model from a .obj file.
/// 
/// Assume all the model files are in the same folder as the .obj file.
/// 
/// use futures-lite instead of async due to deprecation of async in tobj.
/// It might change when refactoring
pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<Model> {
    let parent = std::path::Path::new(file_name).parent().unwrap();
    let obj_text = load_string(file_name).await?;
    let obj_cursor = FuturesCursor::new(obj_text);
    let mut obj_reader = FuturesBufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::futures::load_obj_buf(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            // TODO: Replace unwrap
            let mat_text = load_string(&parent.join(p).into_os_string().into_string().unwrap())
                .await
                .unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        // TODO: Replace unwrap
        let texture_path = parent.join(m.diffuse_texture.unwrap()).into_os_string().into_string().unwrap();
        let diffuse_texture = load_texture(&texture_path, device, queue).await?;
        let normal_texture = load_texture(&m.normal_texture.unwrap(), device, queue).await?;

        materials.push(Material::new(device, &m.name, diffuse_texture, normal_texture, layout));
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| {
                    if m.mesh.normals.is_empty() {
                        ModelVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            tex_coords: [
                                m.mesh.texcoords[i * 2],
                                1.0 - m.mesh.texcoords[i * 2 + 1],
                            ],
                            normal: [0.0, 0.0, 0.0],
                        }
                    } else {
                        ModelVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            tex_coords: [
                                m.mesh.texcoords[i * 2],
                                1.0 - m.mesh.texcoords[i * 2 + 1],
                            ],
                            normal: [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2],
                            ],
                        }
                    }
                })
                .collect::<Vec<_>>();

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            ModelMesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(Model { meshes, materials })
}
