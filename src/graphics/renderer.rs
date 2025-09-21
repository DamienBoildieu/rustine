// Go back to https://sotrh.github.io/learn-wgpu/beginner/tutorial9-models/#accessing-files-from-wasm if adding wasm support is needed

use crate::{
    graphics::{
        Camera, CameraController, DepthTexture, Descriptable, DrawLight, DrawModel, LightUniform, Model, ModelVertex,
    },
    resources,
};

use std::f32::consts;
use std::sync::Arc;

use nalgebra as na;
use winit::window::Window;

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: na::Point3<f32> = na::Point3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

// Will use vertex buffer object for instances as it's done in learn wgpu tutorial. Might be changed later

pub struct Renderer {
    pub config: wgpu::SurfaceConfiguration,
    pub clear_color: wgpu::Color,
    pub camera: Camera,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    is_surface_configured: bool,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: DepthTexture,
    camera_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
    // To move in game
    pub camera_controller: CameraController,
    // Instances
    instances: Vec<na::Matrix4<f32>>,
    instance_buffer: wgpu::Buffer,
    obj_model: Model,
    // Lights
    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    // Could replace window by a trait but it needs to think about the lifetime
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Renderer> {
        let size = window.inner_size();

        // The instance provides a context to create the required structures
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance: wgpu::Instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // Surface represents the window where we will render
        let surface = instance.create_surface(window).unwrap();

        // Adapter is a handle to the GPU hardware
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        // Device is the interface to interact with the GPU
        // Queue is the command queue for submitting work to the GPU
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);

        // Get a compatible sRGB format
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        // The canva configuration
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        /***********************************************************************
        Begin Texture
        ***********************************************************************/
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        /***********************************************************************
        End Texture
        ***********************************************************************/

        /***********************************************************************
        Begin Camera
        ***********************************************************************/
        let camera = Camera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: na::Point3::new(0.0, 5.0, 10.0),
            // have it look at the origin
            target: na::Point3::origin(),
            // which way is "up"
            up: na::Vector3::y_axis(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0f32.to_radians(),
            znear: 0.1,
            zfar: 100.0,
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(camera.build_view_projection_matrix().as_slice()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        /***********************************************************************
        End Camera
        ***********************************************************************/

        /***********************************************************************
        Begin Instance
        ***********************************************************************/
        let obj_model = resources::load_model(
            "models/cube/cube.obj",
            &device,
            &queue,
            &texture_bind_group_layout,
        )
        .await
        .unwrap();

        const SPACE_BETWEEN: f32 = 2.0;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let position = na::Translation3::from(
                        SPACE_BETWEEN * na::Point3::new(x as f32, 0.0, z as f32)
                            - INSTANCE_DISPLACEMENT,
                    );

                    let rotation = if position == na::Translation3::identity() {
                        // this is needed so an object at (0, 0, 0) won't get scaled to zero
                        // as Quaternions can affect scale if they're not created correctly
                        na::UnitQuaternion::from_axis_angle(&na::Vector3::z_axis(), 0.0)
                    } else {
                        na::UnitQuaternion::from_axis_angle(
                            &na::Unit::new_normalize(position.vector),
                            consts::FRAC_PI_4,
                        )
                    };
                    na::Similarity3::from_parts(position, rotation, 1.0).to_homogeneous()
                })
            })
            .collect::<Vec<_>>();

        // For create_buffer_init
        use wgpu::util::DeviceExt;
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        /***********************************************************************
        End Instance
        ***********************************************************************/

        /***********************************************************************
        Begin Depth Buffer
        ***********************************************************************/
        let depth_texture = DepthTexture::new(&device, &config, "depth_texture");

        /***********************************************************************
        End Depth Buffer
        ***********************************************************************/

        /***********************************************************************
        Begin Light
        ***********************************************************************/
        let light_uniform = LightUniform::new([2.0, 2.0, 2.0], [1.0, 1.0, 1.0]);

        // We'll want to update our lights position, so we use COPY_DST
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        // light pipeline/pass
        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../../assets/shaders/light.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(DepthTexture::DEPTH_FORMAT),
                &[ModelVertex::desc()],
                shader,
            )
        };

        /***********************************************************************
        End Light
        ***********************************************************************/

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../assets/shaders/shader.wgsl").into(),
                ),
            };
            create_render_pipeline(
                &device,
                &render_pipeline_layout,
                config.format,
                Some(DepthTexture::DEPTH_FORMAT),
                &[ModelVertex::desc(), na::Matrix4::desc()],
                shader,
            )
        };

        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        let camera_controller = CameraController::new(0.2);

        Ok(Self {
            config,
            clear_color,
            camera,
            surface,
            device,
            queue,
            is_surface_configured: true,
            render_pipeline,
            depth_texture,
            camera_bind_group,
            camera_buffer,
            camera_controller,
            instances,
            instance_buffer,
            obj_model,
            light_uniform,
            light_buffer,
            light_bind_group,
            light_render_pipeline,
        })
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Debug Light model
            render_pass.set_pipeline(&self.light_render_pipeline); // NEW!
            render_pass.draw_light_model(
                &self.obj_model,
                &self.camera_bind_group,
                &self.light_bind_group,
            ); 

            // Main pass
            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            render_pass.set_bind_group(2, &self.light_bind_group, &[]);

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            render_pass.draw_model_instanced(&self.obj_model, 0..self.instances.len() as u32);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
            // TODO: Maybe there is an alternative to recreate the depth texture
            self.depth_texture = DepthTexture::new(&self.device, &self.config, "depth_texture");
        }
    }

    // To move somewhere else
    pub fn update_camera(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(self.camera.build_view_projection_matrix().as_slice()),
        );
    }

    pub fn update_light(&mut self) {
        let old_position: na::Vector3<_> = self.light_uniform.position.into();
        self.light_uniform.position =
            (na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), 1.0f32.to_radians())
                * old_position)
                .into();
        self.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );
    }
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: vertex_layouts,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}
