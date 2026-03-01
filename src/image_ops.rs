use std::path::Path;

use anyhow::Context;
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};

use crate::iconset::IconSpec;

pub fn decode_png(bytes: &[u8]) -> anyhow::Result<DynamicImage> {
    let image = image::load_from_memory_with_format(bytes, ImageFormat::Png)
        .context("OpenAI response was not a PNG image")?;
    Ok(image)
}

pub fn ensure_opaque_square(source: DynamicImage) -> DynamicImage {
    let square = center_crop_square(&source.to_rgba8());
    let flattened = flatten_alpha(&square);
    DynamicImage::ImageRgba8(flattened)
}

pub fn write_icon_set(source: &DynamicImage, specs: &[IconSpec], out_dir: &Path) -> anyhow::Result<()> {
    for spec in specs {
        let resized = source.resize_exact(spec.pixels, spec.pixels, FilterType::Lanczos3);
        let out_path = out_dir.join(&spec.filename);
        resized
            .save_with_format(&out_path, ImageFormat::Png)
            .with_context(|| format!("failed to write {}", out_path.display()))?;
    }
    Ok(())
}

fn flatten_alpha(image: &RgbaImage) -> RgbaImage {
    let bg = pick_background_color(&image);
    let mut out = RgbaImage::new(image.width(), image.height());
    for (x, y, pixel) in image.enumerate_pixels() {
        out.put_pixel(x, y, composite(pixel, bg));
    }
    out
}

fn pick_background_color(image: &RgbaImage) -> [u8; 3] {
    let w = image.width();
    let h = image.height();
    let corners = [
        image.get_pixel(0, 0),
        image.get_pixel(w - 1, 0),
        image.get_pixel(0, h - 1),
        image.get_pixel(w - 1, h - 1),
    ];

    let mut sum = [0u32; 3];
    let mut count = 0u32;
    for px in corners {
        if px[3] > 0 {
            sum[0] += px[0] as u32;
            sum[1] += px[1] as u32;
            sum[2] += px[2] as u32;
            count += 1;
        }
    }

    if count == 0 {
        return [245, 245, 245];
    }
    [
        (sum[0] / count) as u8,
        (sum[1] / count) as u8,
        (sum[2] / count) as u8,
    ]
}

fn composite(fg: &Rgba<u8>, bg: [u8; 3]) -> Rgba<u8> {
    let alpha = fg[3] as f32 / 255.0;
    let inv = 1.0 - alpha;
    let r = (fg[0] as f32 * alpha + bg[0] as f32 * inv).round() as u8;
    let g = (fg[1] as f32 * alpha + bg[1] as f32 * inv).round() as u8;
    let b = (fg[2] as f32 * alpha + bg[2] as f32 * inv).round() as u8;
    Rgba([r, g, b, 255])
}

fn center_crop_square(source: &RgbaImage) -> RgbaImage {
    let width = source.width();
    let height = source.height();
    let side = width.min(height);
    let x = (width - side) / 2;
    let y = (height - side) / 2;
    image::imageops::crop_imm(source, x, y, side, side).to_image()
}

#[cfg(test)]
mod tests {
    use image::{DynamicImage, Rgba, RgbaImage};

    use super::ensure_opaque_square;

    #[test]
    fn makes_square_and_flattens_alpha() {
        let mut img = RgbaImage::new(10, 8);
        for (_, _, px) in img.enumerate_pixels_mut() {
            *px = Rgba([100, 50, 25, 128]);
        }

        let out = ensure_opaque_square(DynamicImage::ImageRgba8(img)).to_rgba8();
        assert_eq!(out.width(), 8);
        assert_eq!(out.height(), 8);
        assert!(out.pixels().all(|p| p[3] == 255));
    }
}
