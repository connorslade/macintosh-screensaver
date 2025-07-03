use std::{fs, path::Path};

use anyhow::{Context, Result};
use bitvec::{order::Lsb0, vec::BitVec};
use image::{GenericImageView, ImageReader};
use nalgebra::Vector2;

use crate::animation::{
    colormap::Colormap, config::AnimationConfig, properties::Properties,
    timeline::PropertiesTimeline,
};

pub mod colormap;
pub mod config;
pub mod properties;
pub mod timeline;

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
    pub image: Image,
    pub timeline: PropertiesTimeline,
}

pub struct Image {
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
                image: Image {
                    data: buffer.into_vec(),
                    size: Vector2::new(image.width(), image.height()),
                },
                timeline: PropertiesTimeline::new(&scene.keyframes),
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

    pub fn scene(&mut self, time: f32) -> (Properties, &Image) {
        let t = time - self.scene_timer.offset;

        let scene_config = &self.config.scenes.scene[self.scene_timer.index];
        let scene_data = &self.scenes[self.scene_timer.index];
        let default = &self.config.scenes.properties;

        if t > scene_config.duration {
            self.scene_timer.offset = time;
            self.scene_timer.index = (self.scene_timer.index + 1) % self.scenes.len();
            self.keyframe = 0;
        }

        let animated = scene_data.timeline.get(t);
        let properties = animated
            .combine(&scene_config.properties)
            .with_defaults(&default);
        (properties, &scene_data.image)
    }

    pub fn scenes(&self) -> usize {
        self.scenes.len()
    }

    pub fn image(&self, n: usize) -> &Image {
        &self.scenes[n].image
    }
}
