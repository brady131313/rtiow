use std::io::Write;

use crate::{interval::Interval, vec::Vec3};

pub type Color = Vec3;

pub const INTENSITY: Interval = Interval::new(0.000, 0.999);

impl Color {
    pub fn write_color<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        let mut r = self.x();
        let mut g = self.y();
        let mut b = self.z();

        r = linear_to_gamma(r);
        g = linear_to_gamma(g);
        b = linear_to_gamma(b);

        let rbyte = (256.0 * INTENSITY.clamp(r)) as i32;
        let gbyte = (256.0 * INTENSITY.clamp(g)) as i32;
        let bbyte = (256.0 * INTENSITY.clamp(b)) as i32;

        writeln!(out, "{rbyte} {gbyte} {bbyte}")
    }
}

fn linear_to_gamma(linear_component: f64) -> f64 {
    if linear_component > 0.0 {
        linear_component.sqrt()
    } else {
        0.0
    }
}
