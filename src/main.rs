use crate::{
    outside_camera::{GpuOutsideCamera, OutsideCamera},
    surface_camera::{GpuSurfaceCamera, SurfaceCamera},
};
use bytemuck::NoUninit;
use eframe::{egui, egui_wgpu::WgpuSetupCreateNew, wgpu};
use math::{Rotor, Vector2, Vector3, Vector4};
use std::{f32::consts::TAU, sync::Arc, time::Instant};

pub mod outside_camera;
pub mod sdf;
pub mod surface_camera;

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
struct RenderSettings {
    signed_distance: u32,
    hit_offset: f32,
    pattern_scale: f32,
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

    outside_texture_width: u32,
    outside_texture_height: u32,
    outside_texture: wgpu::TextureView,
    outside_texture_id: egui::TextureId,
    outside_texture_bind_group: wgpu::BindGroup,

    outside_camera: OutsideCamera,
    outside_camera_buffer: wgpu::Buffer,
    outside_camera_bind_group: wgpu::BindGroup,

    surface_texture_width: u32,
    surface_texture_height: u32,
    surface_texture: wgpu::TextureView,
    surface_texture_id: egui::TextureId,
    surface_texture_bind_group: wgpu::BindGroup,

    surface_camera: SurfaceCamera,
    surface_camera_buffer: wgpu::Buffer,
    surface_camera_bind_group: wgpu::BindGroup,

    plane_height: f32,
    join_position: f32,
    render_settings: RenderSettings,
    objects_info_buffer: wgpu::Buffer,

    render_settings_buffer: wgpu::Buffer,

    wormholes: Vec<Wormhole>,
    wormholes_buffer: wgpu::Buffer,

    spheres: Vec<Sphere>,
    spheres_buffer: wgpu::Buffer,

    objects_bind_group_layout: wgpu::BindGroupLayout,
    objects_bind_group: wgpu::BindGroup,

    outside_ray_tracing_pipeline: wgpu::ComputePipeline,
    surface_ray_tracing_pipeline: wgpu::ComputePipeline,

    cameras_window_open: bool,
    render_settings_window_open: bool,
    wormholes_window_open: bool,
    spheres_window_open: bool,
    outside_view_window_open: bool,
    surface_view_window_open: bool,
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
    render_settings_buffer: &wgpu::Buffer,
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
            wgpu::BindGroupEntry {
                binding: 3,
                resource: render_settings_buffer.as_entire_binding(),
            },
        ],
    })
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

        let outside_texture_width = 1;
        let outside_texture_height = 1;
        let (outside_texture, outside_texture_bind_group) = output_texture_and_bind_group(
            device,
            &output_texture_bind_group_layout,
            outside_texture_width,
            outside_texture_height,
        );
        let outside_texture_id = renderer.write().register_native_texture(
            device,
            &outside_texture,
            wgpu::FilterMode::Nearest,
        );

        let outside_camera = OutsideCamera::new(Vector4 {
            x: -3.0,
            y: 0.0,
            z: 0.0,
            w: 6.0,
        });
        let outside_camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Outside Camera Buffer"),
            size: size_of::<GpuOutsideCamera>().next_multiple_of(16) as _,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let outside_camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Outside Camera Bind Group Layout"),
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
        let outside_camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Outside Camera Bind Group"),
            layout: &outside_camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: outside_camera_buffer.as_entire_binding(),
            }],
        });

        let surface_texture_width = 1;
        let surface_texture_height = 1;
        let (surface_texture, surface_texture_bind_group) = output_texture_and_bind_group(
            device,
            &output_texture_bind_group_layout,
            surface_texture_width,
            surface_texture_height,
        );
        let surface_texture_id = renderer.write().register_native_texture(
            device,
            &surface_texture,
            wgpu::FilterMode::Nearest,
        );

        let surface_camera = SurfaceCamera::new(Vector4 {
            x: 6.0,
            y: 2.0,
            z: 0.0,
            w: 4.0,
        });
        let surface_camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Surface Camera Buffer"),
            size: size_of::<GpuSurfaceCamera>().next_multiple_of(16) as _,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let surface_camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Surface Camera Bind Group Layout"),
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
        let surface_camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Surface Camera Bind Group"),
            layout: &surface_camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: surface_camera_buffer.as_entire_binding(),
            }],
        });

        let objects_info_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Objects Info Buffer"),
            size: size_of::<ObjectsInfo>().next_multiple_of(16) as _,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let render_settings_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rendering Setttings Buffer"),
            size: size_of::<RenderSettings>().next_multiple_of(16) as _,
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
            corner_radius: 3.0,
            padding: [0.0; 3],
        }];
        let wormholes_buffer = wormholes_buffer(device, wormholes.len());

        let spheres = vec![
            Sphere {
                position: Vector4 {
                    x: 8.0,
                    y: 0.0,
                    z: 0.0,
                    w: 6.0,
                },
                rotation: Rotor::identity(),
            },
            Sphere {
                position: Vector4 {
                    x: 8.0,
                    y: 0.0,
                    z: 0.0,
                    w: -6.0,
                },
                rotation: Rotor::rotate_xw(TAU * 0.5),
            },
            Sphere {
                position: Vector4 {
                    x: 3.0,
                    y: 0.0,
                    z: 0.0,
                    w: 0.0,
                },
                rotation: Rotor::identity(),
            },
            Sphere {
                position: Vector4 {
                    x: 10.0,
                    y: 0.0,
                    z: 0.0,
                    w: 0.0,
                },
                rotation: Rotor::identity(),
            },
        ];
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
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
            &render_settings_buffer,
        );

        let outside_ray_tracing_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("OUT_DIR"),
            "/shaders/outside_ray_tracing.wgsl"
        )));
        let outside_ray_tracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Outside Ray Tracing Pipeline Layout"),
                bind_group_layouts: &[
                    &output_texture_bind_group_layout,
                    &outside_camera_bind_group_layout,
                    &objects_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let outside_ray_tracing_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Outside Ray Tracing Pipeline"),
                layout: Some(&outside_ray_tracing_pipeline_layout),
                module: &outside_ray_tracing_shader,
                entry_point: Some("trace_rays"),
                compilation_options: Default::default(),
                cache: None,
            });

        let surface_ray_tracing_shader = device.create_shader_module(wgpu::include_wgsl!(concat!(
            env!("OUT_DIR"),
            "/shaders/surface_ray_tracing.wgsl"
        )));
        let surface_ray_tracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Surface Ray Tracing Pipeline Layout"),
                bind_group_layouts: &[
                    &output_texture_bind_group_layout,
                    &surface_camera_bind_group_layout,
                    &objects_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let surface_ray_tracing_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Surface Ray Tracing Pipeline"),
                layout: Some(&surface_ray_tracing_pipeline_layout),
                module: &surface_ray_tracing_shader,
                entry_point: Some("trace_rays"),
                compilation_options: Default::default(),
                cache: None,
            });

        Self {
            last_time: None,

            output_texture_bind_group_layout,

            outside_texture_width,
            outside_texture_height,
            outside_texture,
            outside_texture_id,
            outside_texture_bind_group,

            surface_texture_width,
            surface_texture_height,
            surface_texture,
            surface_texture_id,
            surface_texture_bind_group,

            outside_camera,
            outside_camera_buffer,
            outside_camera_bind_group,

            surface_camera,
            surface_camera_buffer,
            surface_camera_bind_group,

            plane_height: 4.0,
            join_position: 10.0,
            render_settings: RenderSettings {
                signed_distance: 0,
                hit_offset: 0.0,
                pattern_scale: 50.0,
            },
            objects_info_buffer,

            render_settings_buffer,

            wormholes,
            wormholes_buffer,

            spheres,
            spheres_buffer,

            objects_bind_group_layout,
            objects_bind_group,

            outside_ray_tracing_pipeline,
            surface_ray_tracing_pipeline,

            cameras_window_open: true,
            render_settings_window_open: false,
            wormholes_window_open: false,
            spheres_window_open: false,
            outside_view_window_open: true,
            surface_view_window_open: true,
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
                            .magnitude()
                            + wormhole.throat_size,
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
        cut_plane(
            Vector2 {
                x: p.x - join_position - plane_height,
                y: p.w,
            },
            plane_height,
            plane_height,
        )
    }

    fn project_spheres(&mut self) {
        for sphere in &mut self.spheres {
            let distance = Self::wormholes_sdf(
                &self.wormholes,
                sphere.position,
                self.plane_height,
                self.join_position,
            );

            let normal = sdf::normal(
                |p| Self::wormholes_sdf(&self.wormholes, p, self.plane_height, self.join_position),
                sphere.position,
            );
            sphere.position -= normal * distance;

            if normal.square_magnitude() > 0.0 {
                let old_normal = sphere.rotation.w();
                let correction_rotation =
                    Rotor::from_to_vector(old_normal, normal * old_normal.dot(normal).signum());
                sphere.rotation = correction_rotation.then(sphere.rotation).normalised();
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

        egui::TopBottomPanel::top("Windows").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.cameras_window_open |= ui.button("Cameras").clicked();
                self.render_settings_window_open |= ui.button("Render Settings").clicked();
                self.wormholes_window_open |= ui.button("Wormholes").clicked();
                self.spheres_window_open |= ui.button("Spheres").clicked();
                self.outside_view_window_open |= ui.button("Outside View").clicked();
                self.surface_view_window_open |= ui.button("Surface View").clicked();
            });
        });

        egui::Window::new("Cameras")
            .open(&mut self.cameras_window_open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("FPS: {:.3}", 1.0 / dt.as_secs_f32()));
                ui.collapsing("Outside Camera", |ui| {
                    self.outside_camera.ui(ui);
                });
                ui.collapsing("Surface Camera", |ui| {
                    self.surface_camera.ui(ui);
                });
            });

        egui::Window::new("Wormholes")
            .open(&mut self.wormholes_window_open)
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
                    ui.label("Hit Offset: ");
                    ui.add(egui::DragValue::new(&mut self.render_settings.hit_offset).speed(0.1));
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
                                // wormhole.position.x = wormhole.position.x.min(
                                //     self.join_position
                                //         - self.plane_height
                                //         - wormhole.corner_radius
                                //         - wormhole.throat_size,
                                // );
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

        egui::Window::new("Render Settings")
            .open(&mut self.render_settings_window_open)
            .resizable(false)
            .show(ctx, |ui| {
                egui::Grid::new("Render Settings Grid").show(ui, |ui| {
                    ui.label("Use Signed Distance: ");
                    let mut checked = self.render_settings.signed_distance != 0;
                    ui.add(egui::Checkbox::new(&mut checked, ""));
                    self.render_settings.signed_distance = checked as u8 as u32;
                    ui.end_row();
                    ui.label("Hit Offset: ");
                    ui.add(egui::DragValue::new(&mut self.render_settings.hit_offset).speed(0.1));
                    ui.end_row();
                    ui.label("Pattern Scale: ");
                    ui.add(
                        egui::DragValue::new(&mut self.render_settings.pattern_scale).speed(0.1),
                    );
                    self.render_settings.pattern_scale =
                        self.render_settings.pattern_scale.max(1.0);
                    ui.end_row();
                });
            });

        let mut editing_spheres = false;

        egui::Window::new("Spheres")
            .open(&mut self.spheres_window_open)
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

        egui::Window::new("Outside View")
            .open(&mut self.outside_view_window_open)
            .frame(egui::Frame::window(&ctx.style()).inner_margin(0))
            .show(ctx, |ui| {
                let response = ui.allocate_response(ui.available_size(), egui::Sense::all());

                let width = response.rect.width() as u32;
                let height = response.rect.height() as u32;
                if width > 0
                    && height > 0
                    && width != self.outside_texture_width
                    && height != self.outside_texture_height
                {
                    self.outside_texture_width = width;
                    self.outside_texture_height = height;
                    (self.outside_texture, self.outside_texture_bind_group) =
                        output_texture_and_bind_group(
                            device,
                            &self.output_texture_bind_group_layout,
                            self.outside_texture_width,
                            self.outside_texture_height,
                        );
                    renderer.write().update_egui_texture_from_wgpu_texture(
                        device,
                        &self.outside_texture,
                        wgpu::FilterMode::Nearest,
                        self.outside_texture_id,
                    );
                }

                ui.painter().image(
                    self.outside_texture_id,
                    response.rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 1.0), egui::pos2(1.0, 0.0)),
                    egui::Color32::WHITE,
                );

                if response.hovered() {
                    self.outside_camera.update(ctx, dt.as_secs_f32());
                }
            });

        egui::Window::new("Surface View")
            .open(&mut self.surface_view_window_open)
            .frame(egui::Frame::window(&ctx.style()).inner_margin(0))
            .show(ctx, |ui| {
                let response = ui.allocate_response(ui.available_size(), egui::Sense::all());

                let width = response.rect.width() as u32;
                let height = response.rect.height() as u32;
                if width > 0
                    && height > 0
                    && width != self.surface_texture_width
                    && height != self.surface_texture_height
                {
                    self.surface_texture_width = width;
                    self.surface_texture_height = height;
                    (self.surface_texture, self.surface_texture_bind_group) =
                        output_texture_and_bind_group(
                            device,
                            &self.output_texture_bind_group_layout,
                            self.surface_texture_width,
                            self.surface_texture_height,
                        );
                    renderer.write().update_egui_texture_from_wgpu_texture(
                        device,
                        &self.surface_texture,
                        wgpu::FilterMode::Nearest,
                        self.surface_texture_id,
                    );
                }

                ui.painter().image(
                    self.surface_texture_id,
                    response.rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 1.0), egui::pos2(1.0, 0.0)),
                    egui::Color32::WHITE,
                );

                let sdf = |p| {
                    Self::wormholes_sdf(&self.wormholes, p, self.plane_height, self.join_position)
                };
                self.surface_camera.project(sdf);
                if response.hovered() {
                    self.surface_camera.update(ctx, dt.as_secs_f32(), sdf);
                }
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                _ = ui;
            });

        {
            if self.outside_view_window_open {
                queue.write_buffer(
                    &self.outside_camera_buffer,
                    0,
                    bytemuck::bytes_of(&self.outside_camera.to_gpu()),
                );
            }
            if self.surface_view_window_open {
                queue.write_buffer(
                    &self.surface_camera_buffer,
                    0,
                    bytemuck::bytes_of(&self.surface_camera.to_gpu()),
                );
            }

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

            queue.write_buffer(
                &self.render_settings_buffer,
                0,
                bytemuck::bytes_of(&self.render_settings),
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
                    &self.render_settings_buffer,
                );
            }
        }

        if self.outside_view_window_open || self.surface_view_window_open {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });
            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Compute Pass"),
                    timestamp_writes: None,
                });

                compute_pass.set_bind_group(2, &self.objects_bind_group, &[]);

                if self.outside_view_window_open {
                    compute_pass.set_pipeline(&self.outside_ray_tracing_pipeline);
                    compute_pass.set_bind_group(0, &self.outside_texture_bind_group, &[]);
                    compute_pass.set_bind_group(1, &self.outside_camera_bind_group, &[]);
                    compute_pass.dispatch_workgroups(
                        self.outside_texture_width.div_ceil(16),
                        self.outside_texture_height.div_ceil(16),
                        1,
                    );
                }

                if self.surface_view_window_open {
                    compute_pass.set_pipeline(&self.surface_ray_tracing_pipeline);
                    compute_pass.set_bind_group(0, &self.surface_texture_bind_group, &[]);
                    compute_pass.set_bind_group(1, &self.surface_camera_bind_group, &[]);
                    compute_pass.dispatch_workgroups(
                        self.surface_texture_width.div_ceil(16),
                        self.surface_texture_height.div_ceil(16),
                        1,
                    );
                }
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
