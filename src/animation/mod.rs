use std::{
    env,
    fs::{self, File},
    path::Path,
};

use anyhow::{Context, Result};
use bitvec::{order::Lsb0, vec::BitVec};
use clap::Parser;
use image::{GenericImageView, ImageReader};
use nalgebra::Vector2;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::animation::{
    colormap::Colormap,
    config::AnimationConfig,
    properties::{OptionalProperties, Properties},
    timeline::PropertiesTimeline,
};

pub mod colormap;
pub mod config;
pub mod properties;
pub mod timeline;

#[derive(Serialize, Deserialize)]
pub struct Animation {
    pub colormap: Colormap,
    pub scenes: Vec<SceneData>,
    pub defaults: Properties,

    #[serde(skip)]
    pub scene_timer: Timer,
    #[serde(skip)]
    pub keyframe: usize,
    #[serde(skip)]
    pub runtime: RuntimeConfig,
}

#[derive(Parser, Default)]
#[cfg_attr(windows, command(ignore_errors = true))]
pub struct RuntimeConfig {
    #[arg(long, default_value_t = 0.0)]
    pub fade_duration: f32,
    #[arg(long)]
    pub fade_in: bool,
    #[arg(long)]
    pub fade_out: Option<f32>,
    #[arg(long, alias = "s")]
    pub full_screen: bool,
    #[arg(long, alias = "p")]
    pub preview: Option<isize>,
    #[arg(long, alias = "c", hide = true)]
    pub configure: bool,
}

#[derive(Default)]
pub struct Timer {
    index: usize,
    offset: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SceneData {
    pub frames: Vec<Image>,
    pub duration: f32,
    pub properties: OptionalProperties,
    pub timeline: PropertiesTimeline,
}

#[derive(Serialize, Deserialize)]
pub struct Image {
    pub data: Vec<u32>,
    pub size: Vector2<u32>,
}

impl Animation {
    pub fn load(data: &[u8]) -> Result<Self> {
        let mut this =
            bincode::serde::decode_from_slice::<Self, _>(data, bincode::config::standard())?.0;
        this.scene_timer = Timer::new(this.scenes());
        Ok(this)
    }

    pub fn load_dev(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let dir = path.parent().context("Path must be a file")?;

        let config = fs::read_to_string(path)?;
        let config = toml::from_str::<AnimationConfig>(&config)?;

        let background = ImageReader::open(dir.join(&config.background.colormap))?
            .with_guessed_format()?
            .decode()?;
        let colormap = Colormap::new(background);

        let mut scenes = Vec::with_capacity(config.scenes.scene.len());
        for scene in config.scenes.scene {
            let mut frames = Vec::new();
            let image = ImageReader::open(dir.join(&scene.image))?
                .with_guessed_format()?
                .decode()?;

            let height = image.height() / scene.frames;
            for frame in 0..scene.frames {
                let mut buffer = BitVec::<u32, Lsb0>::new();
                for y in (height * frame)..(height * (frame + 1)) {
                    for x in 0..image.width() {
                        let pixel = image.get_pixel(x, y);
                        let active = pixel[0] != 0 && pixel[1] != 0 && pixel[2] != 0;
                        buffer.push(active);
                    }
                }

                frames.push(Image {
                    data: buffer.into_vec(),
                    size: Vector2::new(image.width(), height),
                });
            }

            scenes.push(SceneData {
                frames,
                duration: scene.duration,
                properties: scene.properties,
                timeline: PropertiesTimeline::new(&scene.keyframes),
            });
        }

        Ok(Self {
            scene_timer: Timer::new(scenes.len()),
            keyframe: 0,

            colormap,
            scenes,
            defaults: config.scenes.properties,
            runtime: RuntimeConfig::default(),
        })
    }

    pub fn runtime_from_args(self) -> Self {
        let args = env::args().flat_map(|x| {
            if let Some(arg) = x.strip_prefix('/') {
                if let Some((key, value)) = arg.split_once(':') {
                    vec![format!("--{key}"), value.to_string()]
                } else {
                    vec![format!("--{arg}")]
                }
            } else {
                vec![x]
            }
        });

        Self {
            runtime: RuntimeConfig::parse_from(args),
            ..self
        }
    }

    pub fn with_runtime(self, runtime: RuntimeConfig) -> Self {
        Self { runtime, ..self }
    }

    pub fn export(&self, path: impl AsRef<Path>) -> Result<()> {
        bincode::serde::encode_into_std_write(
            self,
            &mut File::create(path)?,
            bincode::config::standard(),
        )?;
        Ok(())
    }
}

impl Animation {
    pub fn scene(&mut self, time: f32) -> (Properties, &Image) {
        let t = time - self.scene_timer.offset;
        let scene = &self.scenes[self.scene_timer.index];

        if t > scene.duration {
            self.scene_timer.offset = time;
            self.scene_timer.index = (self.scene_timer.index + 1) % self.scenes.len();
            self.keyframe = 0;
        }

        let animated = scene.timeline.get(t);
        let properties = animated
            .combine(&scene.properties)
            .with_defaults(&self.defaults);
        let frame = &scene.frames[properties.frame % scene.frames.len()];
        (properties, frame)
    }

    pub fn scenes(&self) -> usize {
        self.scenes.len()
    }

    pub fn frames(&self, n: usize) -> usize {
        self.scenes[n].frames.len()
    }

    pub fn image(&self, n: usize, frame: usize) -> &Image {
        &self.scenes[n].frames[frame]
    }
}

impl Timer {
    fn new(max: usize) -> Self {
        let mut rng = rand::rng();
        Timer {
            index: rng.random_range(0..max),
            offset: 0.0,
        }
    }
}
