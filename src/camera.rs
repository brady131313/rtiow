use std::io::Write;

use rand::Rng;

use crate::{
    color::Color,
    hittable::Hittable,
    interval::Interval,
    ray::Ray,
    vec::{Point3, Vec3},
};

pub struct CameraBuilder {
    aspect_ratio: f64,
    image_width: i32,
    samples_per_pixel: i32,
}

impl Default for CameraBuilder {
    fn default() -> Self {
        Self {
            aspect_ratio: 1.0,
            image_width: 100,
            samples_per_pixel: 10,
        }
    }
}

impl CameraBuilder {
    pub fn aspect_ratio(mut self, aspect_ratio: f64) -> Self {
        self.aspect_ratio = aspect_ratio;
        self
    }

    pub fn image_width(mut self, image_width: i32) -> Self {
        self.image_width = image_width;
        self
    }

    pub fn samples_per_pixel(mut self, samples_per_pixel: i32) -> Self {
        self.samples_per_pixel = samples_per_pixel;
        self
    }

    pub fn build(self) -> Camera {
        // Calculate image height, bounded below by 1
        let image_height = ((self.image_width as f64 / self.aspect_ratio) as i32).max(1);

        let pixel_samples_scale = 1.0 / self.samples_per_pixel as f64;

        let focal_length = 1.0;
        let viewport_height = 2.0;
        // Don't use aspect_ratio because we need real valued aspect ratio
        let viewport_width = viewport_height * (self.image_width as f64 / image_height as f64);
        let center = Point3::ZERO;

        // Calculate vectors across horizontal and down vertical viewport edges
        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);

        // Calculate horizontal and vertical delta vectors from pixel to pixel
        let pixel_delta_u = &viewport_u / self.image_width as f64;
        let pixel_delta_v = &viewport_v / image_height as f64;

        // Calculate the location of the upper left pixel
        let viewport_upper_left =
            &center - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00_loc = viewport_upper_left + 0.5 * (&pixel_delta_u + &pixel_delta_v);

        Camera {
            image_height,
            image_width: self.image_width,
            samples_per_pixel: self.samples_per_pixel,
            pixel_samples_scale,
            center,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
        }
    }
}

pub struct Camera {
    image_height: i32,
    image_width: i32,
    samples_per_pixel: i32,
    pixel_samples_scale: f64,
    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
}

impl Camera {
    pub fn builder() -> CameraBuilder {
        CameraBuilder::default()
    }

    pub fn render<H: Hittable, W: Write, E: Write>(
        &self,
        world: &H,
        out: &mut W,
        err_out: &mut E,
    ) -> std::io::Result<()> {
        writeln!(out, "P3\n{} {}\n255", self.image_width, self.image_height)?;
        for j in 0..self.image_height {
            write!(err_out, "\rScanlines remaining: {} ", self.image_height - j)?;

            for i in 0..self.image_width {
                let mut pixel_color = Color::ZERO;
                for _sample in 0..self.samples_per_pixel {
                    let r = self.get_ray(i, j);
                    pixel_color += &self.ray_color(&r, world);
                }

                pixel_color *= self.pixel_samples_scale;
                pixel_color.write_color(out)?;
                // let pixel_center = &self.pixel00_loc
                //     + (i as f64 * &self.pixel_delta_u)
                //     + (j as f64 * &self.pixel_delta_v);
                // let ray_direction = pixel_center - &self.center;

                // let r = Ray::new(self.center.clone(), ray_direction);
                // let pixel_color = self.ray_color(&r, world);
                // pixel_color.write_color(out)?;
            }
        }

        writeln!(err_out, "\rDone.                         ")
    }

    /// Construct a camera ray originating from the origin and directed
    /// at randomly sampled point around the pixel location i, j.
    fn get_ray(&self, i: i32, j: i32) -> Ray {
        let offset = sample_square();
        let pixel_sample = &self.pixel00_loc
            + ((i as f64 + offset.x()) * &self.pixel_delta_u)
            + ((j as f64 + offset.y()) * &self.pixel_delta_v);

        let ray_origin = &self.center;
        let ray_direction = pixel_sample - ray_origin;
        Ray::new(ray_origin.clone(), ray_direction)
    }

    fn ray_color<H: Hittable>(&self, r: &Ray, world: &H) -> Color {
        if let Some(rec) = world.hit(r, Interval::new(0.0, f64::INFINITY)) {
            return 0.5 * (rec.normal + Color::new(1.0, 1.0, 1.0));
        }

        let unit_direction = r.direction().unit_vector();
        let a = 0.5 * (unit_direction.y() + 1.0);

        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }
}

/// Returns the vector to a random point in the [-.5,-.5]-[+.5,+.5] unit square
fn sample_square() -> Vec3 {
    let mut rand = rand::rng();
    Vec3::new(rand.random::<f64>() - 0.5, rand.random::<f64>() - 0.5, 0.0)
}
