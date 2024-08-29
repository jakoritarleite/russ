use std::error::Error;
use std::sync::Arc;

use image::DynamicImage;
use image::GenericImageView;
use image::ImageReader;
use winit::dpi::PhysicalSize;
use winit::raw_window_handle::DisplayHandle;
use winit::window::Window;

use crate::render::Drawable;

pub enum Background {
    Image {
        image: DynamicImage,
        original: Option<DynamicImage>,
    },
    #[allow(dead_code)]
    SolidColor((u8, u8, u8)),
}

impl Background {
    pub fn new_image(path: impl Into<String>) -> Result<Self, Box<dyn Error>> {
        let path = path.into();
        let image = ImageReader::open(&path)?.decode()?;

        Ok(Self::Image {
            image,
            original: None,
        })
    }

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
    fn draw(
        &mut self,
        _window: &Window,
        buffer: &mut softbuffer::Buffer<DisplayHandle<'static>, Arc<Window>>,
    ) -> Result<(), Box<dyn Error>> {
        match self {
            Background::Image { image, original: _ } => {
                let width = image.width() as usize;

                for (x, y, pixel) in image.pixels() {
                    let red = pixel.0[0] as u32;
                    let green = pixel.0[1] as u32;
                    let blue = pixel.0[2] as u32;

                    let color = blue | (green << 8) | (red << 16);

                    buffer[y as usize * width + x as usize] = color;
                }
            }
            Background::SolidColor((r, g, b)) => {
                buffer.fill(u32::from_be_bytes([0x00, *r, *g, *b]));
            }
        }

        Ok(())
    }
}
