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
        resized_image_buffer: Vec<u8>,
    },
    Color(Color),
}

impl Background {
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let Self::Image {
            image,
            resized_image_buffer,
        } = self
        else {
            return;
        };

        let resized = image.resize_to_fill(
            size.width,
            size.height,
            image::imageops::FilterType::Lanczos3,
        );

        // The image contains WxH pixels, where each pixel is 32-bit long
        let mut rgba_data = Vec::with_capacity((resized.width() * resized.height() * 4) as usize);

        for (_, _, pixel) in resized.pixels() {
            let [red, green, blue, alpha] = pixel.0;

            rgba_data.push(red);
            rgba_data.push(green);
            rgba_data.push(blue);
            rgba_data.push(alpha);
        }

        *resized_image_buffer = rgba_data;
    }
}

impl Drawable for Background {
    fn draw(&mut self, _window: &Window, buffer: &mut Pixmap) -> Result<(), DrawError> {
        match self {
            Background::Image {
                image: _,
                resized_image_buffer,
            } => {
                buffer.data_mut().copy_from_slice(resized_image_buffer);
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
                    // that's an "optimization" that instead of crating the resized_image_buffer from
                    // the original image and rewriting it after, we just ignore it here so it can
                    // be populated after. its pretty dumb, but hey, it works.
                    resized_image_buffer: Vec::new(),
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
