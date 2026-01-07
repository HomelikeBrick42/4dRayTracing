use crate::camera::{Camera, GpuCamera};
use bytemuck::NoUninit;
use eframe::{egui, egui_wgpu::WgpuSetupCreateNew, wgpu};
use math::{Rotor, Vector2, Vector3, Vector4};
use std::{sync::Arc, time::Instant};

pub mod camera;
pub mod sdf;

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
struct ObjectsInfo {
    wormholes_count: u32,
    spheres_count: u32,
    plane_height: f32,
    join_position: f32,
}

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
struct Wormhole {
    position: Vector3<f32>,
    throat_size: f32,
    corner_radius: f32,
    padding: [f32; 3],
}

#[derive(Debug)]
struct Sphere {
    position: Vector4<f32>,
    rotation: Rotor,
}

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
struct GpuSphere {
    position: Vector4<f32>,
    forward: Vector4<f32>,
    up: Vector4<f32>,
    right: Vector4<f32>,
    ana: Vector4<f32>,
}

struct App {
    last_time: Option<Instant>,

    output_texture_bind_group_layout: wgpu::BindGroupLayout,

    output_texture_width: u32,
    output_texture_height: u32,
    output_texture: wgpu::TextureView,
    output_texture_id: egui::TextureId,
    output_texture_bind_group: wgpu::BindGroup,

    camera: Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    plane_height: f32,
    join_position: f32,
    objects_info_buffer: wgpu::Buffer,

    wormholes: Vec<Wormhole>,
    wormholes_buffer: wgpu::Buffer,

    spheres: Vec<Sphere>,
    spheres_buffer: wgpu::Buffer,

    objects_bind_group_layout: wgpu::BindGroupLayout,
    objects_bind_group: wgpu::BindGroup,

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

fn wormholes_buffer(device: &wgpu::Device, count: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Wormholes Buffer"),
        size: (count.max(1) * size_of::<Wormhole>()) as _,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn spheres_buffer(device: &wgpu::Device, count: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Spheres Buffer"),
        size: (count.max(1) * size_of::<GpuSphere>()) as _,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn objects_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    objects_info_buffer: &wgpu::Buffer,
    wormholes_buffer: &wgpu::Buffer,
    spheres_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Objects Bind Group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: objects_info_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wormholes_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: spheres_buffer.as_entire_binding(),
            },
        ],
    })
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let eframe::egui_wgpu::RenderState {
            device, renderer, ..
        } = cc.wgpu_render_state.as_ref().unwrap();

        let output_texture_width = 1;
        let output_texture_height = 1;
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
        let (output_texture, output_texture_bind_group) = output_texture_and_bind_group(
            device,
            &output_texture_bind_group_layout,
            output_texture_width,
            output_texture_height,
        );
        let output_texture_id = renderer.write().register_native_texture(
            device,
            &output_texture,
            wgpu::FilterMode::Nearest,
        );

        let camera = Camera::new(Vector4 {
            x: -3.0,
            y: 0.0,
            z: 0.0,
            w: 2.0,
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

        let objects_info_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Objects Info Buffer"),
            size: size_of::<ObjectsInfo>().next_multiple_of(16) as _,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let wormholes = vec![Wormhole {
            position: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            throat_size: 3.0,
            corner_radius: 1.0,
            padding: [0.0; 3],
        }];
        let wormholes_buffer = wormholes_buffer(device, wormholes.len());

        let spheres = vec![Sphere {
            position: Vector4 {
                x: 8.0,
                y: 0.0,
                z: 0.0,
                w: 6.0,
            },
            rotation: Rotor::identity(),
        }];
        let spheres_buffer = spheres_buffer(device, spheres.len());

        let objects_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Objects Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let objects_bind_group = objects_bind_group(
            device,
            &objects_bind_group_layout,
            &objects_info_buffer,
            &wormholes_buffer,
            &spheres_buffer,
        );

        let ray_tracing_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("OUT_DIR"),
            "/shaders/ray_tracing.wgsl"
        )));
        let ray_tracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracing Pipeline Layout"),
                bind_group_layouts: &[
                    &output_texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &objects_bind_group_layout,
                ],
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

            output_texture_width,
            output_texture_height,
            output_texture,
            output_texture_id,
            output_texture_bind_group,

            camera,
            camera_buffer,
            camera_bind_group,

            plane_height: 1.0,
            join_position: 8.0,
            objects_info_buffer,

            wormholes,
            wormholes_buffer,

            spheres,
            spheres_buffer,

            objects_bind_group_layout,
            objects_bind_group,

            ray_tracing_pipeline,
        }
    }

    // fn wormholes_sdf(wormholes: &[Wormhole], p: Vector4<f32>) -> f32 {
    //     let throat_length = 4.0;
    //     let plane = f32::abs(p.w) - throat_length;

    //     let mut d = plane;
    //     for wormhole in wormholes {
    //         let cylinder = (Vector3 {
    //             x: p.x,
    //             y: p.y,
    //             z: p.z,
    //         } - wormhole.position)
    //             .magnitude()
    //             - (wormhole.throat_size + throat_length);
    //         d = f32::max(d, -cylinder);
    //     }
    //     for wormhole in wormholes {
    //         let torus = sdf::torus(
    //             p - Vector4 {
    //                 x: wormhole.position.x,
    //                 y: wormhole.position.y,
    //                 z: wormhole.position.z,
    //                 w: 0.0,
    //             },
    //             wormhole.throat_size + throat_length,
    //             throat_length,
    //         );
    //         d = f32::min(d, torus);
    //     }
    //     d
    // }
    fn wormholes_sdf(
        wormholes: &[Wormhole],
        p: Vector4<f32>,
        plane_height: f32,
        join_position: f32,
    ) -> f32 {
        fn cut_plane(p: Vector2<f32>, plane_height: f32, smooth: f32) -> f32 {
            let d = Vector2 {
                x: p.x + smooth,
                y: p.y.abs() - plane_height + smooth,
            };
            d.max(0.0).magnitude() + d.x.max(d.y).min(0.0) - smooth
        }

        let mut distance = f32::MAX;
        let mut in_wormhole = false;
        for wormhole in wormholes {
            if (Vector3 {
                x: p.x,
                y: p.y,
                z: p.z,
            } - wormhole.position)
                .magnitude()
                < wormhole.throat_size + wormhole.corner_radius + plane_height
            {
                distance = distance.min(cut_plane(
                    Vector2 {
                        x: -(Vector3 {
                            x: p.x,
                            y: p.y,
                            z: p.z,
                        } - wormhole.position)
                            .magnitude() + wormhole.throat_size,
                        y: p.w,
                    },
                    plane_height,
                    wormhole.corner_radius,
                ));
                in_wormhole = true;
            }
        }

        if in_wormhole {
            return distance;
        }
        cut_plane(Vector2 { x: p.x - join_position - plane_height, y: p.w }, plane_height, plane_height)
    }

    fn project_spheres(&mut self) {
        for sphere in &mut self.spheres {
            let distance = Self::wormholes_sdf(&self.wormholes, sphere.position, self.plane_height, self.join_position);
            if f32::abs(distance) < 0.0001 {
                continue;
            }

            let normal = sdf::normal(|p| Self::wormholes_sdf(&self.wormholes, p, self.plane_height, self.join_position), sphere.position);
            sphere.position -= normal * distance;

            if normal.square_magnitude() > 0.0 {
                let old_normal = sphere.rotation.w();
                let correction_rotation = Rotor::from_to_vector(old_normal, normal);
                sphere.rotation = sphere.rotation.then(correction_rotation).normalised();
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        let eframe::egui_wgpu::RenderState {
            device,
            queue,
            renderer,
            ..
        } = frame.wgpu_render_state().unwrap();

        let time = Instant::now();
        let dt = time - self.last_time.unwrap_or(time);
        self.last_time = Some(time);

        egui::Window::new("Camera")
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("FPS: {:.3}", 1.0 / dt.as_secs_f32()));
                self.camera.ui(ui);
            });

        egui::Window::new("Wormholes")
            .resizable(false)
            .show(ctx, |ui| {
                egui::Grid::new("Wormhole Grid").show(ui, |ui| {
                    ui.label("Plane Height: ");
                    ui.add(egui::DragValue::new(&mut self.plane_height).speed(0.1));
                    self.plane_height = self.plane_height.max(0.0);
                    ui.end_row();
                    ui.label("Join Position: ");
                    ui.add(egui::DragValue::new(&mut self.join_position).speed(0.1));
                    ui.end_row();
                });
                if ui.button("New Wormhole").clicked() {
                    self.wormholes.push(Wormhole {
                        position: Vector3 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        throat_size: 3.0,
                        corner_radius: 1.0,
                        padding: [0.0; 3],
                    });
                }

                let mut to_delete = vec![];
                for (i, wormhole) in self.wormholes.iter_mut().enumerate() {
                    ui.push_id(i, |ui| {
                        ui.collapsing("Wormhole", |ui| {
                            egui::Grid::new("Wormhole Grid").show(ui, |ui| {
                                ui.label("Position:");
                                ui.add(
                                    egui::DragValue::new(&mut wormhole.position.x)
                                        .prefix("x:")
                                        .speed(0.1),
                                );
                                wormhole.position.x = wormhole.position.x.min(
                                    self.join_position
                                        - self.plane_height
                                        - wormhole.corner_radius
                                        - wormhole.throat_size,
                                );
                                ui.add(
                                    egui::DragValue::new(&mut wormhole.position.y)
                                        .prefix("y:")
                                        .speed(0.1),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut wormhole.position.z)
                                        .prefix("z:")
                                        .speed(0.1),
                                );
                                ui.end_row();

                                ui.label("Throat Size:");
                                ui.add(egui::DragValue::new(&mut wormhole.throat_size).speed(0.1));
                                wormhole.throat_size = wormhole.throat_size.max(0.0);
                                ui.end_row();

                                ui.label("Corner Radius:");
                                ui.add(
                                    egui::DragValue::new(&mut wormhole.corner_radius).speed(0.1),
                                );
                                wormhole.corner_radius =
                                    wormhole.corner_radius.clamp(0.0, self.plane_height);
                                ui.end_row();

                                if ui.button("Delete").clicked() {
                                    to_delete.push(i);
                                }
                            });
                        });
                    });
                }
                for i in to_delete.into_iter().rev() {
                    self.wormholes.remove(i);
                }
            });

        let mut editing_spheres = false;

        egui::Window::new("Spheres")
            .resizable(false)
            .show(ctx, |ui| {
                if ui.button("New Sphere").clicked() {
                    self.spheres.push(Sphere {
                        position: Vector4 {
                            x: 8.0,
                            y: 0.0,
                            z: 0.0,
                            w: 6.0,
                        },
                        rotation: Rotor::identity(),
                    });
                }

                let mut to_delete = vec![];
                for (i, sphere) in self.spheres.iter_mut().enumerate() {
                    ui.push_id(i, |ui| {
                        ui.collapsing("Sphere", |ui| {
                            egui::Grid::new("Sphere Grid").show(ui, |ui| {
                                ui.label("Position:");
                                editing_spheres |= ui
                                    .add(
                                        egui::DragValue::new(&mut sphere.position.x)
                                            .prefix("x:")
                                            .speed(0.1),
                                    )
                                    .dragged();
                                editing_spheres |= ui
                                    .add(
                                        egui::DragValue::new(&mut sphere.position.y)
                                            .prefix("y:")
                                            .speed(0.1),
                                    )
                                    .dragged();
                                editing_spheres |= ui
                                    .add(
                                        egui::DragValue::new(&mut sphere.position.z)
                                            .prefix("z:")
                                            .speed(0.1),
                                    )
                                    .dragged();
                                editing_spheres |= ui
                                    .add(
                                        egui::DragValue::new(&mut sphere.position.w)
                                            .prefix("w:")
                                            .speed(0.1),
                                    )
                                    .dragged();
                                ui.end_row();
                            });

                            ui.collapsing("Orientation", |ui| {
                                if ui.button("Reset Orientation").clicked() {
                                    sphere.rotation = Rotor::identity();
                                }

                                ui.add_enabled_ui(false, |ui| {
                                    egui::Grid::new("Orientation").show(ui, |ui| {
                                        {
                                            let mut forward = sphere.rotation.x();

                                            ui.label("Forward:");
                                            ui.add(
                                                egui::DragValue::new(&mut forward.x).prefix("x:"),
                                            );
                                            ui.add(
                                                egui::DragValue::new(&mut forward.y).prefix("y:"),
                                            );
                                            ui.add(
                                                egui::DragValue::new(&mut forward.z).prefix("z:"),
                                            );
                                            ui.add(
                                                egui::DragValue::new(&mut forward.w).prefix("w:"),
                                            );
                                            ui.end_row();
                                        }
                                        {
                                            let mut up = sphere.rotation.y();

                                            ui.label("Up:");
                                            ui.add(egui::DragValue::new(&mut up.x).prefix("x:"));
                                            ui.add(egui::DragValue::new(&mut up.y).prefix("y:"));
                                            ui.add(egui::DragValue::new(&mut up.z).prefix("z:"));
                                            ui.add(egui::DragValue::new(&mut up.w).prefix("w:"));
                                            ui.end_row();
                                        }
                                        {
                                            let mut right = sphere.rotation.z();

                                            ui.label("Right:");
                                            ui.add(egui::DragValue::new(&mut right.x).prefix("x:"));
                                            ui.add(egui::DragValue::new(&mut right.y).prefix("y:"));
                                            ui.add(egui::DragValue::new(&mut right.z).prefix("z:"));
                                            ui.add(egui::DragValue::new(&mut right.w).prefix("w:"));
                                            ui.end_row();
                                        }
                                        {
                                            let mut ana = sphere.rotation.w();

                                            ui.label("Ana:");
                                            ui.add(egui::DragValue::new(&mut ana.x).prefix("x:"));
                                            ui.add(egui::DragValue::new(&mut ana.y).prefix("y:"));
                                            ui.add(egui::DragValue::new(&mut ana.z).prefix("z:"));
                                            ui.add(egui::DragValue::new(&mut ana.w).prefix("w:"));
                                            ui.end_row();
                                        }
                                    });
                                });
                            });

                            if ui.button("Delete").clicked() {
                                to_delete.push(i);
                            }
                        });
                    });
                }
                for i in to_delete.into_iter().rev() {
                    self.spheres.remove(i);
                }
            });

        if !editing_spheres {
            self.project_spheres();
        }

        self.camera.update(ctx, dt.as_secs_f32());

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let response = ui.allocate_response(ui.available_size(), egui::Sense::all());

                let width = response.rect.width() as u32;
                let height = response.rect.height() as u32;
                if width > 0
                    && height > 0
                    && width != self.output_texture_width
                    && height != self.output_texture_height
                {
                    self.output_texture_width = width;
                    self.output_texture_height = height;
                    (self.output_texture, self.output_texture_bind_group) =
                        output_texture_and_bind_group(
                            device,
                            &self.output_texture_bind_group_layout,
                            self.output_texture_width,
                            self.output_texture_height,
                        );
                    renderer.write().update_egui_texture_from_wgpu_texture(
                        device,
                        &self.output_texture,
                        wgpu::FilterMode::Nearest,
                        self.output_texture_id,
                    );
                }

                ui.painter().image(
                    self.output_texture_id,
                    response.rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 1.0), egui::pos2(1.0, 0.0)),
                    egui::Color32::WHITE,
                );
            });

        {
            // Camera
            queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::bytes_of(&self.camera.to_gpu()),
            );

            let mut objects_resized = false;

            queue.write_buffer(
                &self.objects_info_buffer,
                0,
                bytemuck::bytes_of(&ObjectsInfo {
                    wormholes_count: self.wormholes.len() as _,
                    spheres_count: self.spheres.len() as _,
                    plane_height: self.plane_height,
                    join_position: self.join_position,
                }),
            );

            if self.wormholes.len() * size_of::<Wormhole>() > self.wormholes_buffer.size() as _ {
                self.wormholes_buffer = wormholes_buffer(device, self.wormholes.len());
                objects_resized = true;
            }
            queue.write_buffer(
                &self.wormholes_buffer,
                0,
                bytemuck::cast_slice(&self.wormholes),
            );

            if self.spheres.len() * size_of::<GpuSphere>() > self.spheres_buffer.size() as _ {
                self.spheres_buffer = spheres_buffer(device, self.spheres.len());
                objects_resized = true;
            }
            queue.write_buffer(
                &self.spheres_buffer,
                0,
                bytemuck::cast_slice(
                    &self
                        .spheres
                        .iter()
                        .map(|sphere| GpuSphere {
                            position: sphere.position,
                            forward: sphere.rotation.x(),
                            up: sphere.rotation.y(),
                            right: sphere.rotation.z(),
                            ana: sphere.rotation.w(),
                        })
                        .collect::<Vec<_>>(),
                ),
            );

            if objects_resized {
                self.objects_bind_group = objects_bind_group(
                    device,
                    &self.objects_bind_group_layout,
                    &self.objects_info_buffer,
                    &self.wormholes_buffer,
                    &self.spheres_buffer,
                );
            }
        }

        {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });
            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Compute Pass"),
                    timestamp_writes: None,
                });

                compute_pass.set_pipeline(&self.ray_tracing_pipeline);
                compute_pass.set_bind_group(0, &self.output_texture_bind_group, &[]);
                compute_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                compute_pass.set_bind_group(2, &self.objects_bind_group, &[]);
                compute_pass.dispatch_workgroups(
                    self.output_texture_width.div_ceil(16),
                    self.output_texture_height.div_ceil(16),
                    1,
                );
            }
            queue.submit(core::iter::once(encoder.finish()));
        }

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
