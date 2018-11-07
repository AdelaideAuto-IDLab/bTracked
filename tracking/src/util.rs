use rand::{self, Rng};
use glm;

const TAU: f32 = 2.0 * ::std::f32::consts::PI;

/// Calculate a random velocity with a given speed
pub fn rand_velocity(speed: f32) -> glm::Vec3 {
    let angle = rand::thread_rng().gen::<f32>() * TAU;
    glm::vec3(speed * angle.cos(), speed * angle.sin(), 0.0)
}

pub fn wrap_angle(angle: f32) -> f32 {
    let wrapped = glm::modf(angle + glm::pi::<f32>(), glm::two_pi::<f32>());
    if wrapped < 0.0 { wrapped + glm::pi::<f32>() } else { wrapped - glm::pi::<f32>() }
}
