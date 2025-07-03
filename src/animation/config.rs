use std::path::PathBuf;

use nalgebra::Vector3;
use serde::Deserialize;

use crate::{
    animation::properties::{OptionalProperties, Properties},
    interpolate::Interpolate,
};

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
    pub image: PathBuf,
    pub duration: f32,

    #[serde(flatten)]
    pub properties: OptionalProperties,
    pub camera_keyframes: Timeline<Vector3<f32>>,
    pub progress_keyframes: Timeline<f32>,
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct Timeline<T> {
    pub keyframes: Vec<Keyframe<T>>,
}

#[derive(Deserialize, Debug)]
pub struct Keyframe<T> {
    pub t: f32,
    pub value: T,
}

impl<T: Interpolate> Timeline<T> {
    pub fn get(&self, t: f32) -> T {
        // todo: sort once by t
        for i in 0..self.keyframes.len() {
            let keyframe = &self.keyframes[i];
            if t < keyframe.t {
                let last = &self.keyframes[i - 1];
                let frac = (t - last.t) / (keyframe.t - last.t);
                return last.value.interpolate(&keyframe.value, frac);
            }
        }

        panic!()
    }
}
