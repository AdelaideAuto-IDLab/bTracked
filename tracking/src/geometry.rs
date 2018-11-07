use std::f32;

use na::{Vector2, Rotation2, Isometry2};
use ncollide2d::{shape::{Cuboid, Capsule, Ball}, query::{self, contacts_internal::support_map_against_support_map}};

use { GeometryConfig, Rect };

pub struct World {
    pub width: f32,
    pub height: f32,
    pub offset: [f32; 2],
    pub scale: f32,
    walls: Vec<(Capsule<f32>, Isometry2<f32>)>,
    obstacles: Vec<(Cuboid<f32>, Isometry2<f32>)>,
}

impl World {
    pub fn new(config: &GeometryConfig) -> World {
        let scale = 1.0 / config.scale;
        let Rect { x: dx, y: dy, width, height } = config.boundary;

        let mut walls = Vec::with_capacity(config.walls.len());
        for wall in &config.walls {
            let a = Vector2::new((wall[0][0] - dx) * scale, (wall[0][1] - dy) * scale);
            let b = Vector2::new((wall[1][0] - dx) * scale, (wall[1][1] - dy) * scale);
            let wall_shape = Capsule::new(0.5 * (a - b).norm(), 0.1);

            let angle = -Rotation2::rotation_between(&(a - b), &Vector2::y()).angle();
            let isometry = Isometry2::new(a + 0.5 * (b - a), angle);

            walls.push((wall_shape, isometry));
        }

        let mut obstacles = Vec::with_capacity(config.obstacles.len());
        for obstacle in &config.obstacles {
            let size = scale * Vector2::new(obstacle.width, obstacle.height);
            let top_left = scale * Vector2::new(obstacle.x - dx, obstacle.y - dy);
            obstacles.push((
                Cuboid::new(0.5 * size + Vector2::new(0.1, 0.1)),
                Isometry2::new(top_left + 0.5 * size, 0.0)
            ));
        }

        World {
            width: width * scale,
            height: height * scale,
            scale: scale,
            offset: [dx, dy],
            walls,
            obstacles,
        }
    }

    /// Determines the optimal direction to travel to get away from the closest wall scaled by the
    /// distance to the wall.
    // TODO: consider finding the optimal direction to travel given the composite polygon to better
    // handle corners. This is difficult in the general case, however we can probably get away with
    // determining the minimal translational distance of a proximity circle around the target point
    pub fn closest_wall(&self, x: f32, y: f32) -> Vector2<f32> {
        let point = Isometry2::new(Vector2::new(x, y), 0.0);
        let query_ball = Ball::new(0.2);

        let mut normal = Vector2::new(0.0, 0.0);
        let mut max_depth: f32 = 0.0;

        for wall in &self.walls {
            match support_map_against_support_map(&point, &query_ball, &wall.1, &wall.0, 0.3) {
                Some(ref contact) if contact.depth > max_depth => {
                    normal = contact.normal.unwrap();
                    max_depth = contact.depth;
                },
                _ => {}
            }
        }

        for obstacle in &self.obstacles {
            match query::contact(&point, &query_ball, &obstacle.1, &obstacle.0, 0.3) {
                Some(ref contact) if contact.depth > max_depth => {
                    normal = contact.normal.unwrap();
                    max_depth = contact.depth;
                },
                _ => {}
            }
        }

        normal * (max_depth.min(0.3) / 0.3).powi(4)
    }
}
