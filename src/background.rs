use std::io;

use image::DynamicImage;
use image::GenericImageView;
use image::ImageReader;
use thiserror::Error;
use tiny_skia::Color;
use tiny_skia::Pixmap;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::config;
use crate::render::DrawError;
use crate::render::Drawable;

pub enum Background {
    Image {
        image: DynamicImage,
        original: Option<DynamicImage>,
    },
    Color(Color),
}

impl Background {
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        // TODO refactor this piece of shit
        if let Self::Image { image, original } = self {
            let img = match original {
                Some(original) => original,
                None => image,
            };

            let resized = img.resize_to_fill(
                size.width,
                size.height,
                image::imageops::FilterType::Nearest,
            );

            if original.is_none() {
                *original = Some(image.clone());
            }

            *image = resized;
        }
    }
}

impl Drawable for Background {
    fn draw(&mut self, _window: &Window, buffer: &mut Pixmap) -> Result<(), DrawError> {
        match self {
            Background::Image { image, original: _ } => {
                // TODO: instead of keeping both original and image instances, keep only the
                // original one and save the rgba_data (resized_image_buffer), so we don't need to
                // do this processing every time.
                let inner_buf = buffer.data_mut();

                // The image contains WxH pixels, where each pixel is 32-bit long
                let mut rgba_data =
                    Vec::with_capacity((image.width() * image.height() * 4) as usize);

                for (_, _, pixel) in image.pixels() {
                    let [red, green, blue, alpha] = pixel.0;

                    rgba_data.push(red);
                    rgba_data.push(green);
                    rgba_data.push(blue);
                    rgba_data.push(alpha);
                }

                // TODO should I premultiply alpha?
                inner_buf.copy_from_slice(&rgba_data);
            }
            Background::Color(color) => {
                buffer.fill(*color);
            }
        }

        Ok(())
    }
}

impl TryFrom<&config::Background> for Background {
    type Error = BackgroundConversionError;

    fn try_from(value: &config::Background) -> Result<Self, Self::Error> {
        match value {
            config::Background::Image(path) => {
                let image = ImageReader::open(path)?.decode()?;

                Ok(Self::Image {
                    image,
                    original: None,
                })
            }
            config::Background::Color((r, g, b)) => {
                Ok(Self::Color(Color::from_rgba8(*r, *g, *b, 0xFF)))
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum BackgroundConversionError {
    #[error("could not open image due to: {0}")]
    Io(#[from] io::Error),

    #[error("there was an error processing the image: {0}")]
    Image(#[from] image::error::ImageError),
}
