use bytemuck::NoUninit;
use eframe::egui;
use math::{NoE2Rotor, Rotor, Transform, Vector4};
use std::f32::consts::TAU;

pub struct Camera {
    pub position: Vector4<f32>,
    pub base_rotation: NoE2Rotor,
    pub xw_rotation: f32,

    pub fov: f32,

    pub move_speed: f32,
    pub rotate_speed: f32,
}

impl Camera {
    pub fn new(position: Vector4<f32>) -> Self {
        Self {
            position,
            base_rotation: NoE2Rotor::identity(),
            xw_rotation: 0.0,

            fov: TAU * 0.25,

            move_speed: 1.0,
            rotate_speed: TAU * 0.5,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, ts: f32) {
        if !ctx.wants_keyboard_input() {
            ctx.input(|i| {
                if i.key_down(egui::Key::W) {
                    self.position += self.base_rotation.x() * self.move_speed * ts;
                }
                if i.key_down(egui::Key::S) {
                    self.position -= self.base_rotation.x() * self.move_speed * ts;
                }
                if i.key_down(egui::Key::A) {
                    self.position -= self.base_rotation.z() * self.move_speed * ts;
                }
                if i.key_down(egui::Key::D) {
                    self.position += self.base_rotation.z() * self.move_speed * ts;
                }
                if i.key_down(egui::Key::Q) {
                    self.position -= self.base_rotation.w() * self.move_speed * ts;
                }
                if i.key_down(egui::Key::E) {
                    self.position += self.base_rotation.w() * self.move_speed * ts;
                }
                if i.key_down(egui::Key::R) {
                    self.position += self.base_rotation.y() * self.move_speed * ts;
                }
                if i.key_down(egui::Key::F) {
                    self.position -= self.base_rotation.y() * self.move_speed * ts;
                }

                if i.key_down(egui::Key::ArrowLeft) {
                    self.base_rotation = self
                        .base_rotation
                        .then(NoE2Rotor::rotate_xz(-self.rotate_speed * ts));
                }
                if i.key_down(egui::Key::ArrowRight) {
                    self.base_rotation = self
                        .base_rotation
                        .then(NoE2Rotor::rotate_xz(self.rotate_speed * ts));
                }
                if i.key_down(egui::Key::ArrowUp) {
                    self.xw_rotation += self.rotate_speed * ts;
                }
                if i.key_down(egui::Key::ArrowDown) {
                    self.xw_rotation -= self.rotate_speed * ts;
                }
            });
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("Camera").show(ui, |ui| {
            ui.label("Position:");
            ui.add(egui::DragValue::new(&mut self.position.x).prefix("x:"));
            ui.add(egui::DragValue::new(&mut self.position.y).prefix("y:"));
            ui.add(egui::DragValue::new(&mut self.position.z).prefix("z:"));
            ui.add(egui::DragValue::new(&mut self.position.w).prefix("w:"));
            ui.end_row();

            ui.label("XW Rotation:");
            ui.drag_angle(&mut self.xw_rotation);
            ui.end_row();

            ui.label("Fov:");
            ui.drag_angle(&mut self.fov);
            self.fov = self.fov.clamp(0.0, 179f32.to_radians());
            ui.end_row();

            ui.label("Move Speed:");
            ui.add(egui::DragValue::new(&mut self.move_speed));
            ui.end_row();

            ui.label("Rotate Speed:");
            ui.drag_angle(&mut self.rotate_speed);
            ui.end_row();
        });

        ui.collapsing("Computed Transform", |ui| {
            ui.add_enabled_ui(false, |ui| {
                egui::Grid::new("Computed Transform").show(ui, |ui| {
                    let transform = self.transform();

                    {
                        let mut position = transform.position();

                        ui.label("Position:");
                        ui.add(egui::DragValue::new(&mut position.x).prefix("x:"));
                        ui.add(egui::DragValue::new(&mut position.y).prefix("y:"));
                        ui.add(egui::DragValue::new(&mut position.z).prefix("z:"));
                        ui.add(egui::DragValue::new(&mut position.w).prefix("w:"));
                        ui.end_row();
                    }
                    {
                        let mut forward = transform.x();

                        ui.label("Forward:");
                        ui.add(egui::DragValue::new(&mut forward.x).prefix("x:"));
                        ui.add(egui::DragValue::new(&mut forward.y).prefix("y:"));
                        ui.add(egui::DragValue::new(&mut forward.z).prefix("z:"));
                        ui.add(egui::DragValue::new(&mut forward.w).prefix("w:"));
                        ui.end_row();
                    }
                    {
                        let mut up = transform.w();

                        ui.label("Up:");
                        ui.add(egui::DragValue::new(&mut up.x).prefix("x:"));
                        ui.add(egui::DragValue::new(&mut up.y).prefix("y:"));
                        ui.add(egui::DragValue::new(&mut up.z).prefix("z:"));
                        ui.add(egui::DragValue::new(&mut up.w).prefix("w:"));
                        ui.end_row();
                    }
                    {
                        let mut right = transform.z();

                        ui.label("Right:");
                        ui.add(egui::DragValue::new(&mut right.x).prefix("x:"));
                        ui.add(egui::DragValue::new(&mut right.y).prefix("y:"));
                        ui.add(egui::DragValue::new(&mut right.z).prefix("z:"));
                        ui.add(egui::DragValue::new(&mut right.w).prefix("w:"));
                        ui.end_row();
                    }
                    {
                        let mut ana = transform.y();

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

    pub fn rotation(&self) -> Rotor {
        Rotor::from_no_e2_rotor(self.base_rotation).then(Rotor::rotate_xw(self.xw_rotation))
    }

    pub fn transform(&self) -> Transform {
        Transform::translation(self.position).then(Transform::from_rotor(self.rotation()))
    }

    pub fn to_gpu(&self) -> GpuCamera {
        let transform = self.transform();
        GpuCamera {
            position: transform.position(),
            forward: transform.x(),
            up: transform.w(),
            right: transform.z(),
            fov: self.fov,
        }
    }
}

#[derive(Clone, Copy, NoUninit)]
#[repr(C)]
pub struct GpuCamera {
    position: Vector4<f32>,
    forward: Vector4<f32>,
    up: Vector4<f32>,
    right: Vector4<f32>,
    fov: f32,
}
