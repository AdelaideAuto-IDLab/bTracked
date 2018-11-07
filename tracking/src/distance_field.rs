use rayon::prelude::*;

use geometry::World;

pub struct DistanceField {
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    pub world_scale: f32,
    pub offset: [f32; 2],
    pub data: Vec<[f32; 2]>,
}

impl DistanceField {
    pub fn new(world: &World, scale: f32) -> DistanceField {
        let width = (world.width * scale) as u32;
        let height = (world.height * scale) as u32;

        println!("Generating distance field with -- width: {}, height: {}", width, height);

        let data = (0..height * width).into_par_iter().map(|i| {
            let x = i % width;
            let y = i / width;

            let (map_x, map_y) = (x as f32 / scale, y as f32 / scale);
            let wall_vec = world.closest_wall(map_x, map_y);
            [wall_vec.x, wall_vec.y]
        }).collect();

        DistanceField { width, height, scale, world_scale: world.scale, offset: world.offset, data }
    }

    pub fn query(&self, x: f32, y: f32) -> [f32; 2] {
        if x < 0.0 || y < 0.0 {
            return [0.0; 2];
        }

        let scaled_x = (x * self.scale).round() as u32;
        let scaled_y = (y * self.scale).round() as u32;

        if scaled_x >= self.width || scaled_y >= self.height {
            return [0.0; 2]
        }

        self.data[(scaled_y * self.width + scaled_x) as usize]
    }
}
