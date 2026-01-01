use std::f32::consts::TAU;

use bytemuck::NoUninit;
use eframe::egui;
use math::{NoE2Rotor, Rotor, Transform, Vector4};

pub struct Camera {
    pub position: Vector4<f32>,
    pub base_rotation: NoE2Rotor,
    pub xy_rotation: f32,

    pub speed: f32,
    pub rotation_speed: f32,
}

impl Camera {
    pub fn new(position: Vector4<f32>) -> Self {
        Self {
            position,
            base_rotation: NoE2Rotor::identity(),
            xy_rotation: 0.0,

            speed: 1.0,
            rotation_speed: TAU * 0.5,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, ts: f32) {
        if !ctx.wants_keyboard_input() {
            ctx.input(|i| {
                if i.key_down(egui::Key::W) {
                    self.position += self.base_rotation.x() * self.speed * ts;
                }
                if i.key_down(egui::Key::S) {
                    self.position += self.base_rotation.x() * self.speed * ts;
                }
                if i.key_down(egui::Key::A) {
                    self.position -= self.base_rotation.z() * self.speed * ts;
                }
                if i.key_down(egui::Key::D) {
                    self.position += self.base_rotation.z() * self.speed * ts;
                }
                if i.key_down(egui::Key::Q) {
                    self.position -= self.base_rotation.y() * self.speed * ts;
                }
                if i.key_down(egui::Key::E) {
                    self.position += self.base_rotation.y() * self.speed * ts;
                }
                if i.key_down(egui::Key::R) {
                    self.position += self.base_rotation.w() * self.speed * ts;
                }
                if i.key_down(egui::Key::F) {
                    self.position -= self.base_rotation.w() * self.speed * ts;
                }

                if i.key_down(egui::Key::ArrowLeft) {
                    self.base_rotation = self
                        .base_rotation
                        .then(NoE2Rotor::rotate_xz(-self.rotation_speed * ts));
                }
                if i.key_down(egui::Key::ArrowRight) {
                    self.base_rotation = self
                        .base_rotation
                        .then(NoE2Rotor::rotate_xz(self.rotation_speed * ts));
                }
                if i.key_down(egui::Key::ArrowUp) {
                    self.xy_rotation += self.rotation_speed * ts;
                }
                if i.key_down(egui::Key::ArrowDown) {
                    self.xy_rotation -= self.rotation_speed * ts;
                }
            });
        }
    }

    pub fn rotation(&self) -> Rotor {
        Rotor::from_no_e2_rotor(self.base_rotation).then(Rotor::rotate_xy(self.xy_rotation))
    }

    pub fn transform(&self) -> Transform {
        Transform::translation(self.position).then(Transform::from_rotor(self.rotation()))
    }

    pub fn to_gpu(&self) -> GpuCamera {
        let transform = self.transform();
        GpuCamera {
            position: transform.position(),
            forward: transform.x(),
            up: transform.y(),
            right: transform.z(),
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
}
