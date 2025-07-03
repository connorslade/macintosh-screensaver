use std::{fs, path::Path};

use anyhow::{Context, Result};
use bitvec::{order::Lsb0, vec::BitVec};
use image::{GenericImageView, ImageReader};
use nalgebra::Vector2;

use crate::animation::{
    colormap::Colormap,
    config::AnimationConfig,
    properties::{OptionalProperties, Properties},
};

pub mod colormap;
pub mod config;
pub mod properties;

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

        let camera_pos = scene.camera_keyframes.get(t);
        let progress = scene.progress_keyframes.get(t);

        let properties = OptionalProperties::camera_pos(camera_pos)
            .combine(&OptionalProperties::progress(progress))
            .combine(&scene.properties)
            .with_defaults(&default);
        (properties, &image)
    }
}
