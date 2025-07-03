use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use bitvec::{order::Lsb0, vec::BitVec};
use image::{GenericImageView, ImageReader};
use nalgebra::{Matrix4, Vector2};
use serde::Deserialize;
use tufa::export::nalgebra::Vector3;

use crate::{colormap::Colormap, interpolate::Interpolate};

pub struct Animation {
    pub config: AnimationConfig,

    pub colormap: Colormap,
    pub scenes: Vec<SceneData>,

    pub scene_timer: Timer,
    pub keyframe: usize,
}

#[derive(Default)]
pub struct Timer {
    index: usize,
    offset: f32,
}

#[derive(Deserialize, Debug)]
pub struct AnimationConfig {
    background: BackgroundConfig,
    scenes: ScenesConfig,
}

#[derive(Deserialize, Debug)]
pub struct BackgroundConfig {
    colormap: PathBuf,
    duration: f32,
}

#[derive(Deserialize, Debug)]
pub struct ScenesConfig {
    #[serde(flatten)]
    properties: Properties,
    scene: Vec<SceneConfig>,
}

#[derive(Deserialize, Debug)]
pub struct SceneConfig {
    image: PathBuf,
    duration: f32,

    keyframes: Vec<Keyframe>,
}

#[derive(Deserialize, Debug)]
pub struct Keyframe {
    t: f32,
    #[serde(flatten)]
    properties: OptionalProperties,
}

#[derive(Deserialize, Debug)]
pub struct OptionalProperties {
    camera_pos: Option<Vector3<f32>>,
    camera_dir: Option<Vector3<f32>>,
    scale: Option<f32>,
    progress: Option<f32>,
    progress_angle: Option<f32>,
}

#[derive(Deserialize, Debug)]
pub struct Properties {
    pub camera_pos: Vector3<f32>,
    pub camera_dir: Vector3<f32>,
    pub scale: f32,
    pub progress: f32,
    pub progress_angle: f32,
}

pub struct SceneData {
    pub data: Vec<u32>,
    pub size: Vector2<u32>,
}

impl Animation {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let dir = path.parent().context("Path must be a file")?;

        let config = fs::read_to_string(path)?;
        let config = toml::from_str::<AnimationConfig>(&config)?;

        let background = ImageReader::open(dir.join(&config.background.colormap))?
            .with_guessed_format()?
            .decode()?;
        let colormap = Colormap::new(background);

        let mut scenes = Vec::with_capacity(config.scenes.scene.len());
        for scene in config.scenes.scene.iter() {
            let image = ImageReader::open(dir.join(&scene.image))?
                .with_guessed_format()?
                .decode()?;

            let mut buffer = BitVec::<u32, Lsb0>::new();
            for y in 0..image.height() {
                for x in 0..image.width() {
                    let pixel = image.get_pixel(x, y).0[0] != 0;
                    buffer.push(pixel);
                }
            }

            scenes.push(SceneData {
                data: buffer.into_vec(),
                size: Vector2::new(image.width(), image.height()),
            });
        }

        Ok(Self {
            config,
            colormap,
            scenes,

            scene_timer: Timer::default(),
            keyframe: 0,
        })
    }

    pub fn scene(&mut self, t: f32) -> (Properties, &SceneData) {
        let t = t - self.scene_timer.offset;

        let image = &self.scenes[self.scene_timer.index];
        let default = &self.config.scenes.properties;

        let scene = &self.config.scenes.scene[self.scene_timer.index];
        if t > scene.duration {
            self.scene_timer.offset = t;
            self.scene_timer.index = (self.scene_timer.index + 1) % self.scenes.len();
            self.keyframe = 0;
        }

        let keyframe = &scene.keyframes[self.keyframe];
        let next = &scene.keyframes[self.keyframe + 1];

        if t >= next.t {
            self.keyframe += 1;
        }

        let frac = (t - keyframe.t) / (next.t - keyframe.t);
        let properties = keyframe
            .properties
            .with_defaults(default)
            .interpolate(&next.properties.with_defaults(default), frac);

        (properties, &image)
    }
}

impl OptionalProperties {
    pub fn with_defaults(&self, defaults: &Properties) -> Properties {
        Properties {
            camera_pos: self.camera_pos.unwrap_or(defaults.camera_pos),
            camera_dir: self.camera_dir.unwrap_or(defaults.camera_dir),
            scale: self.scale.unwrap_or(defaults.scale),
            progress: self.progress.unwrap_or(defaults.progress),
            progress_angle: self.progress_angle.unwrap_or(defaults.progress_angle),
        }
    }
}

impl Properties {
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        Properties {
            camera_pos: self.camera_pos.lerp(&other.camera_pos, t),
            camera_dir: self.camera_dir.lerp(&other.camera_dir, t),
            scale: self.scale.interpolate(&other.scale, t),
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
