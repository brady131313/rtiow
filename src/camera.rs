use std::io::Write;

use crate::{
    color::Color,
    hittable::Hittable,
    interval::Interval,
    ray::Ray,
    vec::{Point3, Vec3},
};

pub struct Camera {
    image_height: i32,
    image_width: i32,
    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
}

impl Camera {
    pub fn new(aspect_ratio: f64, image_width: i32) -> Self {
        // Calculate image height, bounded below by 1
        let image_height = ((image_width as f64 / aspect_ratio) as i32).max(1);

        let focal_length = 1.0;
        let viewport_height = 2.0;
        // Don't use aspect_ratio because we need real valued aspect ratio
        let viewport_width = viewport_height * (image_width as f64 / image_height as f64);
        let center = Point3::ZERO;

        // Calculate vectors across horizontal and down vertical viewport edges
        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);

        // Calculate horizontal and vertical delta vectors from pixel to pixel
        let pixel_delta_u = &viewport_u / image_width as f64;
        let pixel_delta_v = &viewport_v / image_height as f64;

        // Calculate the location of the upper left pixel
        let viewport_upper_left =
            &center - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00_loc = viewport_upper_left + 0.5 * (&pixel_delta_u + &pixel_delta_v);

        Self {
            image_height,
            image_width,
            center,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
        }
    }

    pub fn render<H: Hittable>(&self, world: &H) {
        println!("P3\n{} {}\n255", self.image_width, self.image_height);
        for j in 0..self.image_height {
            eprint!("\rScanlines remaining: {} ", self.image_height - j);
            std::io::stderr().flush().unwrap();

            for i in 0..self.image_width {
                let pixel_center = &self.pixel00_loc
                    + (i as f64 * &self.pixel_delta_u)
                    + (j as f64 * &self.pixel_delta_v);
                let ray_direction = pixel_center - &self.center;

                let r = Ray::new(self.center.clone(), ray_direction);
                let pixel_color = self.ray_color(&r, world);
                println!("{pixel_color}");
            }
        }

        eprintln!("\rDone.                         ")
    }

    fn ray_color<H: Hittable>(&self, r: &Ray, world: &H) -> Color {
        if let Some(rec) = world.hit(r, Interval::new(0.0, f64::INFINITY)) {
            let raw_color = 0.5 * (rec.normal + Vec3::new(1.0, 1.0, 1.0));
            return Color::from(raw_color);
        }

        let unit_direction = r.direction().unit_vector();
        let a = 0.5 * (unit_direction.y() + 1.0);
        let raw_color = (1.0 - a) * Vec3::new(1.0, 1.0, 1.0) + a * Vec3::new(0.5, 0.7, 1.0);
        Color::from(raw_color)
    }
}
