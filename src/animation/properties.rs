use nalgebra::{Matrix4, Vector3};
use serde::{Deserialize, Serialize};

use crate::interpolate::Interpolate;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct OptionalProperties {
    pub camera_pos: Option<Vector3<f32>>,
    pub camera_dir: Option<Vector3<f32>>,
    pub scale: Option<f32>,
    pub frame: Option<usize>,
    pub progress: Option<f32>,
    pub progress_angle: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Properties {
    pub camera_pos: Vector3<f32>,
    pub camera_dir: Vector3<f32>,
    pub scale: f32,
    pub frame: usize,
    pub progress: f32,
    pub progress_angle: f32,
}

impl OptionalProperties {
    pub fn combine(&self, other: &Self) -> Self {
        Self {
            camera_pos: self.camera_pos.or(other.camera_pos),
            camera_dir: self.camera_dir.or(other.camera_dir),
            scale: self.scale.or(other.scale),
            frame: self.frame.or(other.frame),
            progress: self.progress.or(other.progress),
            progress_angle: self.progress_angle.or(other.progress_angle),
        }
    }

    pub fn with_defaults(&self, defaults: &Properties) -> Properties {
        Properties {
            camera_pos: self.camera_pos.unwrap_or(defaults.camera_pos),
            camera_dir: self.camera_dir.unwrap_or(defaults.camera_dir),
            scale: self.scale.unwrap_or(defaults.scale),
            frame: self.frame.unwrap_or(defaults.frame),
            progress: self.progress.unwrap_or(defaults.progress),
            progress_angle: self.progress_angle.unwrap_or(defaults.progress_angle),
        }
    }

    pub fn camera_pos(pos: Vector3<f32>) -> Self {
        Self {
            camera_pos: Some(pos),
            ..Default::default()
        }
    }

    pub fn progress(progress: f32) -> Self {
        Self {
            progress: Some(progress),
            ..Default::default()
        }
    }
}

impl Properties {
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        Properties {
            camera_pos: self.camera_pos.lerp(&other.camera_pos, t),
            camera_dir: self.camera_dir.lerp(&other.camera_dir, t),
            scale: self.scale.interpolate(&other.scale, t),
            frame: self.frame.interpolate(&other.frame, t),
            progress: self.progress.interpolate(&other.progress, t),
            progress_angle: self.progress_angle.interpolate(&other.progress_angle, t),
        }
    }

    pub fn view_projection(&self, aspect: f32) -> Matrix4<f32> {
        let depth = 100.0;
        let projection = if aspect < 1.0 {
            Matrix4::new_orthographic(-aspect, aspect, -1.0, 1.0, -depth, depth)
        } else {
            Matrix4::new_orthographic(-1.0, 1.0, -aspect.recip(), aspect.recip(), -depth, depth)
        };

        let scale = Matrix4::new_scaling(self.scale);
        let view = Matrix4::look_at_rh(
            &self.camera_pos.into(),
            &(self.camera_pos + self.camera_dir).into(),
            &Vector3::z_axis(),
        );

        projection * view * scale
    }
}
