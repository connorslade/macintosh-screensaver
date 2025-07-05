use image::{DynamicImage, EncodableLayout, Rgb, RgbImage};
use nalgebra::Vector3;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize)]
pub struct Colormap {
    #[serde(
        serialize_with = "serialize_image",
        deserialize_with = "deserialize_image"
    )]
    inner: RgbImage,
}

const COLUMN_BACKGROUND_TOP: u32 = 0;
const COLUMN_BACGKROUND_BOTTOM: u32 = 1;
const COLUMN_FOREGROUND: u32 = 2;

impl Colormap {
    pub fn new(image: DynamicImage) -> Self {
        Self {
            inner: image.into_rgb8(),
        }
    }

    pub fn get_background_top(&self, t: f32) -> Vector3<f32> {
        self.get_color(COLUMN_BACKGROUND_TOP, t)
    }

    pub fn get_background_bottom(&self, t: f32) -> Vector3<f32> {
        self.get_color(COLUMN_BACGKROUND_BOTTOM, t)
    }

    pub fn get_foreground(&self, t: f32) -> Vector3<f32> {
        self.get_color(COLUMN_FOREGROUND, t)
    }

    fn get_color(&self, x: u32, t: f32) -> Vector3<f32> {
        let height = self.inner.height() as f32;
        let px = height * t;

        let low = self.inner.get_pixel(x, px.floor() as u32);
        let high = self.inner.get_pixel(x, (px.ceil() % height) as u32);

        to_nalgebra(low).lerp(&to_nalgebra(high), px.fract())
    }
}

fn to_nalgebra(color: &Rgb<u8>) -> Vector3<f32> {
    Vector3::new(color.0[0], color.0[1], color.0[2]).map(|x| x as f32 / 255.0)
}

#[derive(Serialize, Deserialize)]
struct RawImage {
    width: u32,
    data: Vec<u8>,
}

fn serialize_image<S: Serializer>(image: &RgbImage, serializer: S) -> Result<S::Ok, S::Error> {
    RawImage {
        width: image.width(),
        data: image.as_bytes().to_owned(),
    }
    .serialize(serializer)
}

fn deserialize_image<'de, D: Deserializer<'de>>(deserializer: D) -> Result<RgbImage, D::Error> {
    let raw = RawImage::deserialize(deserializer)?;
    let height = raw.data.len() as u32 / raw.width / 3;
    let image = RgbImage::from_raw(raw.width, height, raw.data).unwrap();
    Ok(image)
}
