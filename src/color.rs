use std::io::Write;

use crate::{interval::Interval, vec::Vec3};

pub type Color = Vec3;

pub const INTENSITY: Interval = Interval::new(0.000, 0.999);

impl Color {
    pub fn write_color<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        let r = self.x();
        let g = self.y();
        let b = self.z();

        let rbyte = (256.0 * INTENSITY.clamp(r)) as i32;
        let gbyte = (256.0 * INTENSITY.clamp(g)) as i32;
        let bbyte = (256.0 * INTENSITY.clamp(b)) as i32;

        writeln!(out, "{rbyte} {gbyte} {bbyte}")
    }
}
