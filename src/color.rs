use std::fmt::Display;

use crate::vec::Vec3;

pub struct Color(Vec3);

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self(Vec3::new(r, g, b))
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = self.0.x();
        let g = self.0.y();
        let b = self.0.z();

        let rbyte = (255.999 * r) as i32;
        let gbyte = (255.999 * g) as i32;
        let bbyte = (255.999 * b) as i32;

        write!(f, "{rbyte} {gbyte} {bbyte}")
    }
}
