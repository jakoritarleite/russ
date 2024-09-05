use std::io;

use fast_image_resize::images::Image;
use fast_image_resize::IntoImageView;
use fast_image_resize::PixelType;
use fast_image_resize::Resizer;
use image::DynamicImage;
use image::ImageReader;
use image::Luma;
use image::LumaA;
use image::Pixel;
use image::Rgb;
use image::Rgba;
use num_traits::NumCast;
use num_traits::ToPrimitive;
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

        let mut resized = Image::new(size.width, size.height, image.pixel_type().unwrap());
        let mut resizer = Resizer::new();
        resizer.resize(image, &mut resized, None).unwrap();

        // The image contains width * height number of pixels, where each pixel is 32-bit long
        // this way we need a buffer of width * height * 4 (8bit for red, green, blue and alpha)
        let mut rgba_data = Vec::with_capacity((size.width * size.height * 4) as usize);

        let resized_pixel_type = resized.pixel_type();

        //for pixel in resized.pixels() {
        for pixel in resized.buffer().chunks_exact(resized_pixel_type.size()) {
            let [red, green, blue, alpha] = cast_pixel_to_rgba_u8(pixel, resized_pixel_type).0;

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

fn cast_u8_to_u16(v: u8) -> u16 {
    let v = v.to_u64().unwrap();
    NumCast::from((v << 8) | v).unwrap()
}

fn cast_u16_to_u8(v: u16) -> u8 {
    NumCast::from((v as u32 + 128) / 257).unwrap()
}

fn cast_u8_to_f32(v: u8) -> f32 {
    (v as f32 / u8::MAX as f32).clamp(0.0, 1.0)
}

fn cast_f32_to_u8(v: f32) -> u8 {
    NumCast::from((v.clamp(0.0, 1.0) * u8::MAX as f32).round()).unwrap()
}

fn cast_pixel_to_rgba_u8(pixel: &[u8], pixel_type: PixelType) -> Rgba<u8> {
    match pixel_type {
        // Luma8
        PixelType::U8 => Luma::<u8>::from_slice(pixel).to_rgba(),

        // LumaA8
        PixelType::U8x2 => LumaA::<u8>::from_slice(pixel).to_rgba(),

        // Rgb8
        PixelType::U8x3 => Rgb::<u8>::from_slice(pixel).to_rgba(),

        // Rgba8
        PixelType::U8x4 => Rgba::<u8>::from_slice(pixel).to_owned(),

        // Luma16
        PixelType::U16 => {
            let l = [cast_u8_to_u16(pixel[0])];
            let luma16 = Luma::<u16>::from_slice(&l);

            Luma::<u8>::from_slice(&[cast_u16_to_u8(luma16.channels()[0])]).to_rgba()
        }

        // LumaA16
        PixelType::U16x2 => {
            let l = [cast_u8_to_u16(pixel[0]), cast_u8_to_u16(pixel[1])];
            let luma16 = LumaA::<u16>::from_slice(&l);

            LumaA::<u8>::from_slice(&[
                cast_u16_to_u8(luma16.channels()[0]),
                cast_u16_to_u8(luma16.channels()[1]),
            ])
            .to_rgba()
        }

        // Rgb16
        PixelType::U16x3 => {
            let p = [
                cast_u8_to_u16(pixel[0]),
                cast_u8_to_u16(pixel[1]),
                cast_u8_to_u16(pixel[2]),
            ];
            let rgb16 = Rgb::<u16>::from_slice(&p);

            Rgb::<u8>::from_slice(&[
                cast_u16_to_u8(rgb16.channels()[0]),
                cast_u16_to_u8(rgb16.channels()[1]),
                cast_u16_to_u8(rgb16.channels()[2]),
            ])
            .to_rgba()
        }

        // Rgba16
        PixelType::U16x4 => {
            let p = [
                cast_u8_to_u16(pixel[0]),
                cast_u8_to_u16(pixel[1]),
                cast_u8_to_u16(pixel[2]),
                cast_u8_to_u16(pixel[3]),
            ];
            let rgba16 = Rgba::<u16>::from_slice(&p);

            Rgba::<u8>::from_slice(&[
                cast_u16_to_u8(rgba16.channels()[0]),
                cast_u16_to_u8(rgba16.channels()[1]),
                cast_u16_to_u8(rgba16.channels()[2]),
                cast_u16_to_u8(rgba16.channels()[3]),
            ])
            .to_owned()
        }

        // TODO: convert from Luma<i32>
        //PixelType::I32 => {}

        // Luma32f
        PixelType::F32 => {
            let l = [cast_u8_to_f32(pixel[0])];
            let luma32f = Luma::<f32>::from_slice(&l);

            Luma::<u8>::from_slice(&[cast_f32_to_u8(luma32f.channels()[0])]).to_rgba()
        }

        // LumaA32f
        PixelType::F32x2 => {
            let l = [cast_u8_to_f32(pixel[0]), cast_u8_to_f32(pixel[1])];
            let luma32f = LumaA::<f32>::from_slice(&l);

            LumaA::<u8>::from_slice(&[
                cast_f32_to_u8(luma32f.channels()[0]),
                cast_f32_to_u8(luma32f.channels()[1]),
            ])
            .to_rgba()
        }

        // Rgb32f
        PixelType::F32x3 => {
            let l = [
                cast_u8_to_f32(pixel[0]),
                cast_u8_to_f32(pixel[1]),
                cast_u8_to_f32(pixel[2]),
            ];
            let rgb32f = Rgb::<f32>::from_slice(&l);

            Rgb::<u8>::from_slice(&[
                cast_f32_to_u8(rgb32f.channels()[0]),
                cast_f32_to_u8(rgb32f.channels()[1]),
                cast_f32_to_u8(rgb32f.channels()[2]),
            ])
            .to_rgba()
        }

        // Rgba32f
        PixelType::F32x4 => {
            let l = [
                cast_u8_to_f32(pixel[0]),
                cast_u8_to_f32(pixel[1]),
                cast_u8_to_f32(pixel[2]),
                cast_u8_to_f32(pixel[3]),
            ];
            let rgba32f = Rgba::<f32>::from_slice(&l);

            Rgba::<u8>::from_slice(&[
                cast_f32_to_u8(rgba32f.channels()[0]),
                cast_f32_to_u8(rgba32f.channels()[1]),
                cast_f32_to_u8(rgba32f.channels()[2]),
                cast_f32_to_u8(rgba32f.channels()[3]),
            ])
            .to_owned()
        }

        _ => {
            todo!()
        }
    }
}
