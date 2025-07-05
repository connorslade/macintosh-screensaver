use nalgebra::Vector3;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::{
    animation::{config::PropertyKeyframe, properties::OptionalProperties},
    interpolate::Interpolate,
};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(transparent)]
pub struct Timeline<T> {
    pub keyframes: Vec<Keyframe<T>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Keyframe<T> {
    pub t: f32,
    pub value: T,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PropertiesTimeline {
    camera_pos: Timeline<Vector3<f32>>,
    camera_dir: Timeline<Vector3<f32>>,
    scale: Timeline<f32>,
    frame: Timeline<usize>,
    progress: Timeline<f32>,
    progress_angle: Timeline<f32>,
}

impl<T: Interpolate + Copy> Timeline<T> {
    fn sort(&mut self) {
        self.keyframes.sort_by_key(|x| OrderedFloat(x.t));
    }

    pub fn get(&self, t: f32) -> Option<T> {
        for i in 0..self.keyframes.len() {
            let keyframe = &self.keyframes[i];
            if t < keyframe.t {
                let Some(last) = self.keyframes.get(i - 1) else {
                    return Some(keyframe.value);
                };

                let frac = (t - last.t) / (keyframe.t - last.t);
                let out = if frac.is_nan() {
                    keyframe.value
                } else {
                    last.value.interpolate(&keyframe.value, frac)
                };

                return Some(out);
            }
        }

        self.keyframes.last().map(|x| x.value)
    }
}

impl PropertiesTimeline {
    pub fn new(keyframes: &[PropertyKeyframe]) -> Self {
        let mut timeline = Self::default();

        for keyframe in keyframes {
            let t = keyframe.t;
            if let Some(value) = keyframe.properties.camera_dir {
                timeline.camera_dir.keyframes.push(Keyframe { t, value });
            }

            if let Some(value) = keyframe.properties.camera_pos {
                timeline.camera_pos.keyframes.push(Keyframe { t, value });
            }

            if let Some(value) = keyframe.properties.scale {
                timeline.scale.keyframes.push(Keyframe { t, value });
            }

            if let Some(value) = keyframe.properties.frame {
                timeline.frame.keyframes.push(Keyframe { t, value });
            }

            if let Some(value) = keyframe.properties.progress {
                timeline.progress.keyframes.push(Keyframe { t, value });
            }

            if let Some(value) = keyframe.properties.progress_angle {
                timeline
                    .progress_angle
                    .keyframes
                    .push(Keyframe { t, value });
            }
        }

        timeline.camera_dir.sort();
        timeline.camera_pos.sort();
        timeline.scale.sort();
        timeline.frame.sort();
        timeline.progress.sort();
        timeline.progress_angle.sort();

        timeline
    }

    pub fn get(&self, t: f32) -> OptionalProperties {
        OptionalProperties {
            camera_pos: self.camera_pos.get(t),
            camera_dir: self.camera_dir.get(t),
            scale: self.scale.get(t),
            frame: self.frame.get(t),
            progress: self.progress.get(t),
            progress_angle: self.progress_angle.get(t),
        }
    }
}
