use image::{DynamicImage, GenericImageView, Rgba};
use tufa::export::nalgebra::Vector3;

pub struct Colormap {
    inner: DynamicImage,
}

impl Colormap {
    pub fn new(image: DynamicImage) -> Self {
        Self { inner: image }
    }

    pub fn get_background_top(&self, t: f32) -> Vector3<f32> {
        self.get_color(0, t)
    }

    pub fn get_background_bottom(&self, t: f32) -> Vector3<f32> {
        self.get_color(1, t)
    }

    pub fn get_foreground(&self, t: f32) -> Vector3<f32> {
        self.get_color(2, t)
    }

    fn get_color(&self, x: u32, t: f32) -> Vector3<f32> {
        let px = self.inner.height() as f32 * t;
        let frac = px.fract();

        let low = self.inner.get_pixel(x, px.floor() as u32);
        let high = self.inner.get_pixel(x, px.ceil() as u32);

        to_nalgebra(low).lerp(&to_nalgebra(high), frac)
    }
}

fn to_nalgebra(color: Rgba<u8>) -> Vector3<f32> {
    Vector3::new(color.0[0], color.0[1], color.0[2]).map(|x| x as f32 / 255.0)
}
