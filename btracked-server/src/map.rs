use std::io::Cursor;

use image::{self, DynamicImage, ImageBuffer, RgbaImage, ImageFormat};
use na;
use palette::{Hsv, rgb::LinSrgb, RgbHue};

use tracking::{GeometryConfig, geometry::World, distance_field::DistanceField};

fn to_color_image(field: &DistanceField) -> RgbaImage {
    ImageBuffer::from_fn(field.width, field.height, |x, y| {
        let wall_vec = na::Vector2::from(field.data[(y * field.width + x) as usize]);
        let angle = wall_vec.y.atan2(wall_vec.x);

        let color = Hsv::new(RgbHue::from_radians(angle), 1.0, 1.0);
        let (r, g, b) = LinSrgb::from(color).into_components();
        image::Rgba([(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, (wall_vec.norm() * 255.0) as u8])
    })
}

pub fn distance_field_from_png(buffer: &[u8], config: &GeometryConfig) -> Result<DistanceField, String> {
    let image = match image::load_from_memory_with_format(buffer, ImageFormat::PNG) {
        Ok(DynamicImage::ImageRgba8(data)) => data,
        Ok(_) => return Err(format!("Unsupported image format (must be RGBA format")),
        Err(e) => return Err(format!("Error decoding image: {}", e)),
    };

    let (width, height) = image.dimensions();
    let data = image.pixels().map(|pixel| {
        let [r, g, b, a] = pixel.data;
        if a == 0 {
            return [0.0, 0.0];
        }
        let color = LinSrgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);

        let angle = Hsv::from(color).hue.to_radians();
        let vec = na::Vector2::new(angle.cos(), angle.sin()) * (a as f32 / 255.0);
        vec.into()
    }).collect();

    Ok(DistanceField {
        width,
        height,
        scale: 100.0,
        world_scale: config.scale,
        offset: [config.boundary.x, config.boundary.y],
        data
    })
}

pub fn generate_collision_map(config: &GeometryConfig) -> Result<Vec<u8>, String> {
    let world = World::new(config);
    let dist_field = DistanceField::new(&world, 100.0);

    let mut buffer = Cursor::new(vec![]);
    DynamicImage::ImageRgba8(to_color_image(&dist_field))
        .write_to(&mut buffer, ImageFormat::PNG)
        .map_err(|e| format!("{}", e))?;

    Ok(buffer.into_inner())
}
