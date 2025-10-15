use std::path::Path;

use anyhow::Context;
use image::{ConvertColorOptions, ImageReader, RgbImage, metadata::Cicp};

pub struct RtwImage {
    image: RgbImage,
}

impl RtwImage {
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();

        let mut image = ImageReader::open(path)
            .with_context(|| format!("Failed to open image: {path:?}"))?
            .decode()?
            .into_rgb8();

        image
            .apply_color_space(Cicp::SRGB_LINEAR, ConvertColorOptions::default())
            .context("Failed to convert image to linear color space")?;

        Ok(Self { image })
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }

    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> (u8, u8, u8) {
        let pixel = self.image.get_pixel(x, y);
        (pixel.0[0], pixel.0[1], pixel.0[2])
    }
}
