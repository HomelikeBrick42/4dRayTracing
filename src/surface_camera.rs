use crate::sdf;
use bytemuck::NoUninit;
use eframe::egui;
use math::{Rotor, Vector4};
use std::f32::consts::TAU;

pub struct SurfaceCamera {
    pub position: Vector4<f32>,
    pub rotation: Rotor,

    pub fov: f32,

    pub move_speed: f32,
    pub rotate_speed: f32,
}

impl SurfaceCamera {
    pub fn new(position: Vector4<f32>) -> Self {
        Self {
            position,
            rotation: Rotor::identity(),

            fov: TAU * 0.25,

            move_speed: 5.0,
            rotate_speed: TAU * 0.5,
        }
    }

    pub fn project(&mut self, mut f: impl FnMut(Vector4<f32>) -> f32) {
        let distance = f(self.position);
        if f32::abs(distance) < 0.0001 {
            return;
        }

        let normal = sdf::normal(&mut f, self.position);
        self.position -= normal * distance;

        if normal.square_magnitude() > 0.0 {
            let old_normal = self.rotation.w();
            let correction_rotation =
                Rotor::from_to_vector(old_normal, normal * old_normal.dot(normal).signum());
            self.rotation = correction_rotation.then(self.rotation).normalised();
        }
    }

    pub fn walk(
        &mut self,
        direction: Vector4<f32>,
        mut distance: f32,
        mut f: impl FnMut(Vector4<f32>) -> f32,
    ) {
        while distance > 0.0 {
            let step = distance.min(0.01);
            distance -= step;

            self.position += direction * step;
            self.project(&mut f);
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, ts: f32, mut f: impl FnMut(Vector4<f32>) -> f32) {
        if !ctx.wants_keyboard_input() {
            ctx.input(|i| {
                if i.key_down(egui::Key::W) {
                    self.walk(self.rotation.x(), self.move_speed * ts, &mut f);
                }
                if i.key_down(egui::Key::S) {
                    self.walk(-self.rotation.x(), self.move_speed * ts, &mut f);
                }
                if i.key_down(egui::Key::A) {
                    self.walk(-self.rotation.z(), self.move_speed * ts, &mut f);
                }
                if i.key_down(egui::Key::D) {
                    self.walk(self.rotation.z(), self.move_speed * ts, &mut f);
                }
                if i.key_down(egui::Key::Q) {
                    self.walk(-self.rotation.y(), self.move_speed * ts, &mut f);
                }
                if i.key_down(egui::Key::E) {
                    self.walk(self.rotation.y(), self.move_speed * ts, &mut f);
                }

                if i.key_down(egui::Key::ArrowLeft) {
                    self.rotation = self
                        .rotation
                        .then(Rotor::rotate_xz(-self.rotate_speed * ts));
                }
                if i.key_down(egui::Key::ArrowRight) {
                    self.rotation = self.rotation.then(Rotor::rotate_xz(self.rotate_speed * ts));
                }

                if i.key_down(egui::Key::ArrowUp) {
                    self.rotation = self.rotation.then(Rotor::rotate_xy(self.rotate_speed * ts));
                }
                if i.key_down(egui::Key::ArrowDown) {
                    self.rotation = self
                        .rotation
                        .then(Rotor::rotate_xy(-self.rotate_speed * ts));
                }
            });
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("Camera").show(ui, |ui| {
            ui.label("Position:");
            ui.add(
                egui::DragValue::new(&mut self.position.x)
                    .prefix("x:")
                    .speed(0.1),
            );
            ui.add(
                egui::DragValue::new(&mut self.position.y)
                    .prefix("y:")
                    .speed(0.1),
            );
            ui.add(
                egui::DragValue::new(&mut self.position.z)
                    .prefix("z:")
                    .speed(0.1),
            );
            ui.add(
                egui::DragValue::new(&mut self.position.w)
                    .prefix("w:")
                    .speed(0.1),
            );
            ui.end_row();

            ui.label("Fov:");
            ui.drag_angle(&mut self.fov);
            self.fov = self.fov.clamp(0.0, 179f32.to_radians());
            ui.end_row();

            ui.label("Move Speed:");
            ui.add(egui::DragValue::new(&mut self.move_speed).speed(0.1));
            ui.end_row();

            ui.label("Rotate Speed:");
            ui.drag_angle(&mut self.rotate_speed);
            ui.end_row();
        });

        ui.collapsing("Computed Transform", |ui| {
            ui.add_enabled_ui(false, |ui| {
                egui::Grid::new("Computed Transform").show(ui, |ui| {
                    {
                        let mut position = self.position;

                        ui.label("Position:");
                        ui.add(egui::DragValue::new(&mut position.x).prefix("x:"));
                        ui.add(egui::DragValue::new(&mut position.y).prefix("y:"));
                        ui.add(egui::DragValue::new(&mut position.z).prefix("z:"));
                        ui.add(egui::DragValue::new(&mut position.w).prefix("w:"));
                        ui.end_row();
                    }
                    {
                        let mut forward = self.rotation.x();

                        ui.label("Forward:");
                        ui.add(egui::DragValue::new(&mut forward.x).prefix("x:"));
                        ui.add(egui::DragValue::new(&mut forward.y).prefix("y:"));
                        ui.add(egui::DragValue::new(&mut forward.z).prefix("z:"));
                        ui.add(egui::DragValue::new(&mut forward.w).prefix("w:"));
                        ui.end_row();
                    }
                    {
                        let mut up = self.rotation.w();

                        ui.label("Up:");
                        ui.add(egui::DragValue::new(&mut up.x).prefix("x:"));
                        ui.add(egui::DragValue::new(&mut up.y).prefix("y:"));
                        ui.add(egui::DragValue::new(&mut up.z).prefix("z:"));
                        ui.add(egui::DragValue::new(&mut up.w).prefix("w:"));
                        ui.end_row();
                    }
                    {
                        let mut right = self.rotation.z();

                        ui.label("Right:");
                        ui.add(egui::DragValue::new(&mut right.x).prefix("x:"));
                        ui.add(egui::DragValue::new(&mut right.y).prefix("y:"));
                        ui.add(egui::DragValue::new(&mut right.z).prefix("z:"));
                        ui.add(egui::DragValue::new(&mut right.w).prefix("w:"));
                        ui.end_row();
                    }
                    {
                        let mut ana = self.rotation.y();

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
    }

    pub fn to_gpu(&self) -> GpuSurfaceCamera {
        GpuSurfaceCamera {
            position: self.position,
            rotation: self.rotation,
            fov: self.fov,
        }
    }
}

#[derive(Clone, Copy, NoUninit)]
#[repr(C)]
pub struct GpuSurfaceCamera {
    position: Vector4<f32>,
    rotation: Rotor,
    fov: f32,
}
