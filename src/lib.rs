use core::f64;

pub mod camera;
pub mod color;
pub mod hittable;
pub mod interval;
pub mod material;
pub mod ray;
pub mod scene_loader;
pub mod sphere;
pub mod vec;

pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * f64::consts::PI / 180.0
}
