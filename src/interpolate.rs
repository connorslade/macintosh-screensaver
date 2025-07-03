use nalgebra::Vector3;

pub trait Interpolate {
    fn interpolate(&self, other: &Self, t: f32) -> Self;
}

impl Interpolate for f32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self * (1.0 - t) + other * t
    }
}

impl Interpolate for Vector3<f32> {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self.lerp(other, t)
    }
}
