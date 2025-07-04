use std::path::PathBuf;

use serde::Deserialize;

use crate::animation::properties::{OptionalProperties, Properties};

#[derive(Deserialize, Debug)]
pub struct AnimationConfig {
    pub background: BackgroundConfig,
    pub scenes: ScenesConfig,
}

#[derive(Deserialize, Debug)]
pub struct BackgroundConfig {
    pub colormap: PathBuf,
    pub duration: f32,
}

#[derive(Deserialize, Debug)]
pub struct ScenesConfig {
    #[serde(flatten)]
    pub properties: Properties,
    pub scene: Vec<SceneConfig>,
}

#[derive(Deserialize, Debug)]
pub struct SceneConfig {
    pub frames: Vec<PathBuf>,
    #[serde(default = "default_frametime")]
    pub frametime: f32,
    pub duration: f32,

    #[serde(flatten)]
    pub properties: OptionalProperties,
    pub keyframes: Vec<PropertyKeyframe>,
}

#[derive(Deserialize, Debug)]
pub struct PropertyKeyframe {
    pub t: f32,
    #[serde(flatten)]
    pub properties: OptionalProperties,
}

fn default_frametime() -> f32 {
    1.0
}
