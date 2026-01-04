use crate::camera::{Camera, GpuCamera};
use eframe::{egui, egui_wgpu::WgpuSetupCreateNew, wgpu};
use math::Vector4;
use std::{sync::Arc, time::Instant};

pub mod camera;

struct App {
    last_time: Option<Instant>,

    output_texture_bind_group_layout: wgpu::BindGroupLayout,
    output_texture: wgpu::TextureView,
    output_texture_id: egui::TextureId,
    output_texture_bind_group: wgpu::BindGroup,

    camera: Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    ray_tracing_pipeline: wgpu::ComputePipeline,
}

fn output_texture_and_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    width: u32,
    height: u32,
) -> (wgpu::TextureView, wgpu::BindGroup) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Output Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    });
    let texture_view = texture.create_view(&Default::default());

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Texture Bind Group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&texture_view),
        }],
    });

    (texture_view, texture_bind_group)
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let eframe::egui_wgpu::RenderState {
            device, renderer, ..
        } = cc.wgpu_render_state.as_ref().unwrap();

        let output_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Output Texture Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });
        let (output_texture, output_texture_bind_group) =
            output_texture_and_bind_group(device, &output_texture_bind_group_layout, 1, 1);
        let output_texture_id = renderer.write().register_native_texture(
            device,
            &output_texture,
            wgpu::FilterMode::Nearest,
        );

        let camera = Camera::new(Vector4 {
            x: -3.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        });
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: size_of::<GpuCamera>().next_multiple_of(16) as _,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let ray_tracing_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("OUT_DIR"),
            "/shaders/ray_tracing.wgsl"
        )));
        let ray_tracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracing Pipeline Layout"),
                bind_group_layouts: &[&output_texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });
        let ray_tracing_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Ray Tracing Pipeline"),
                layout: Some(&ray_tracing_pipeline_layout),
                module: &ray_tracing_shader,
                entry_point: Some("trace_rays"),
                compilation_options: Default::default(),
                cache: None,
            });

        Self {
            last_time: None,

            output_texture_bind_group_layout,
            output_texture,
            output_texture_id,
            output_texture_bind_group,

            camera,
            camera_buffer,
            camera_bind_group,

            ray_tracing_pipeline,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        let time = Instant::now();
        let dt = time - self.last_time.unwrap_or(time);
        self.last_time = Some(time);

        egui::Window::new("Camera")
            .resizable(false)
            .show(ctx, |ui| {
                self.camera.ui(ui);
            });

        self.camera.update(ctx, dt.as_secs_f32());

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let eframe::egui_wgpu::RenderState {
                    device,
                    queue,
                    renderer,
                    ..
                } = frame.wgpu_render_state().unwrap();

                let response = ui.allocate_response(ui.available_size(), egui::Sense::all());

                let width = response.rect.width() as u32;
                let height = response.rect.height() as u32;
                if width > 0
                    && height > 0
                    && width != self.output_texture.texture().width()
                    && height != self.output_texture.texture().height()
                {
                    (self.output_texture, self.output_texture_bind_group) =
                        output_texture_and_bind_group(
                            device,
                            &self.output_texture_bind_group_layout,
                            width,
                            height,
                        );
                    renderer.write().update_egui_texture_from_wgpu_texture(
                        device,
                        &self.output_texture,
                        wgpu::FilterMode::Nearest,
                        self.output_texture_id,
                    );
                }

                queue.write_buffer(
                    &self.camera_buffer,
                    0,
                    bytemuck::bytes_of(&self.camera.to_gpu()),
                );

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Command Encoder"),
                });
                {
                    let mut compute_pass =
                        encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                            label: Some("Compute Pass"),
                            timestamp_writes: None,
                        });

                    compute_pass.set_pipeline(&self.ray_tracing_pipeline);
                    compute_pass.set_bind_group(0, &self.output_texture_bind_group, &[]);
                    compute_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                    compute_pass.dispatch_workgroups(width.div_ceil(16), height.div_ceil(16), 1);
                }
                queue.submit(core::iter::once(encoder.finish()));

                ui.painter().image(
                    self.output_texture_id,
                    response.rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 1.0), egui::pos2(1.0, 0.0)),
                    egui::Color32::WHITE,
                );
            });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "4d Ray Tracing",
        eframe::NativeOptions {
            vsync: false,
            wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
                present_mode: wgpu::PresentMode::AutoNoVsync,
                wgpu_setup: eframe::egui_wgpu::WgpuSetup::CreateNew(WgpuSetupCreateNew {
                    instance_descriptor: wgpu::InstanceDescriptor::from_env_or_default(),
                    device_descriptor: Arc::new(|adapter| wgpu::DeviceDescriptor {
                        label: Some("Wgpu Device"),
                        required_features: wgpu::Features::BGRA8UNORM_STORAGE,
                        required_limits: adapter.limits(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
