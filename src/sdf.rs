use math::{Vector2, Vector3, Vector4};

pub fn normal(mut f: impl FnMut(Vector4<f32>) -> f32, p: Vector4<f32>) -> Vector4<f32> {
    let x = Vector4 {
        x: 0.001,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    let y = Vector4 {
        x: 0.0,
        y: 0.001,
        z: 0.0,
        w: 0.0,
    };
    let z = Vector4 {
        x: 0.0,
        y: 0.0,
        z: 0.001,
        w: 0.0,
    };
    let w = Vector4 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.001,
    };
    Vector4 {
        x: f(p + x) - f(p - x),
        y: f(p + y) - f(p - y),
        z: f(p + z) - f(p - z),
        w: f(p + w) - f(p - w),
    }
    .normalised()
}

pub fn torus(p: Vector4<f32>, ring_radius: f32, radius: f32) -> f32 {
    let position = Vector4 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    let q = Vector2 {
        x: Vector3 {
            x: p.x - position.x,
            y: p.y - position.y,
            z: p.z - position.z,
        }
        .magnitude()
            - ring_radius,
        y: p.w - position.w,
    };
    q.magnitude() - radius
}
