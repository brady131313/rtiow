use std::io::Write;

use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rand::Rng;

use crate::{
    color::Color,
    degrees_to_radians,
    hittable::{Hittable, HittableList},
    interval::Interval,
    ray::Ray,
    vec::{Point3, Vec3},
};

pub struct CameraBuilder {
    aspect_ratio: f64,
    image_width: i32,
    /// Cound of random samples for each pixel
    samples_per_pixel: i32,
    /// Max number of ray bounces into scene
    max_depth: i32,
    /// Vertical view angle (field of view)
    vfov: f64,
    /// Point camera is looking from
    lookfrom: Point3,
    //// point camera is looking at
    lookat: Point3,
    /// camera-relative up direction
    vup: Vec3,
    /// variation angle of rays through each pixel
    defocus_angle: f64,
    /// distance from camera lookfrom point to plane of perfect focus
    focus_dist: f64,
}

impl Default for CameraBuilder {
    fn default() -> Self {
        Self {
            aspect_ratio: 1.0,
            image_width: 100,
            samples_per_pixel: 10,
            max_depth: 10,
            vfov: 90.0,
            lookfrom: Point3::ZERO,
            lookat: Point3::new(0.0, 0.0, -1.0),
            vup: Vec3::new(0.0, 1.0, 0.0),
            defocus_angle: 0.0,
            focus_dist: 10.0,
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

    pub fn max_depth(mut self, max_depth: i32) -> Self {
        self.max_depth = max_depth;
        self
    }

    pub fn vfov(mut self, vfov: f64) -> Self {
        self.vfov = vfov;
        self
    }

    pub fn lookfrom(mut self, lookfrom: Point3) -> Self {
        self.lookfrom = lookfrom;
        self
    }

    pub fn lookat(mut self, lookat: Point3) -> Self {
        self.lookat = lookat;
        self
    }

    pub fn vup(mut self, vup: Vec3) -> Self {
        self.vup = vup;
        self
    }

    pub fn defocus_angle(mut self, defocus_angle: f64) -> Self {
        self.defocus_angle = defocus_angle;
        self
    }

    pub fn focus_dist(mut self, focus_dist: f64) -> Self {
        self.focus_dist = focus_dist;
        self
    }

    pub fn build(self) -> Camera {
        // Calculate image height, bounded below by 1
        let image_height = ((self.image_width as f64 / self.aspect_ratio) as i32).max(1);

        let pixel_samples_scale = 1.0 / self.samples_per_pixel as f64;
        let center = self.lookfrom.clone();

        let theta = degrees_to_radians(self.vfov);
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * self.focus_dist;
        // Don't use aspect_ratio because we need real valued aspect ratio
        let viewport_width = viewport_height * (self.image_width as f64 / image_height as f64);

        // Calculate the u,v,w unit basis vectors for the camera coordinate frame
        let w = (&self.lookfrom - &self.lookat).unit_vector();
        let u = self.vup.cross(&w).unit_vector();
        let v = w.cross(&u);

        // Calculate vectors across horizontal and down vertical viewport edges
        let viewport_u = viewport_width * &u; // Vector across viewport horizontal edge
        let viewport_v = viewport_height * -&v; // Vector across viewport vertical edge

        // Calculate horizontal and vertical delta vectors from pixel to pixel
        let pixel_delta_u = &viewport_u / self.image_width as f64;
        let pixel_delta_v = &viewport_v / image_height as f64;

        // Calculate the location of the upper left pixel
        let viewport_upper_left =
            &center - (self.focus_dist * &w) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00_loc = viewport_upper_left + 0.5 * (&pixel_delta_u + &pixel_delta_v);

        let defocus_radius = self.focus_dist * degrees_to_radians(self.defocus_angle / 2.0).tan();
        let defocus_disk_u = &u * defocus_radius;
        let defocus_disk_v = &v * defocus_radius;

        Camera {
            image_height,
            image_width: self.image_width,
            samples_per_pixel: self.samples_per_pixel,
            pixel_samples_scale,
            center,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
            max_depth: self.max_depth,
            defocus_angle: self.defocus_angle,
            defocus_disk_u,
            defocus_disk_v,
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
    max_depth: i32,
    // /// Camera frame basis vectors
    // u: Vec3,
    // v: Vec3,
    // w: Vec3,
    defocus_angle: f64,
    /// defocus disk horizontal radius
    defocus_disk_u: Vec3,
    /// defocus disk vertical radius
    defocus_disk_v: Vec3,
}

impl Camera {
    pub fn builder() -> CameraBuilder {
        CameraBuilder::default()
    }

    pub fn render<W: Write>(&self, world: &HittableList, out: &mut W) -> std::io::Result<()> {
        use rayon::prelude::*;

        let pb = ProgressBar::new(self.image_height as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .unwrap()
    );

        let pixels: Vec<_> = (0..self.image_height)
            .into_par_iter()
            .progress_with(pb.clone())
            .flat_map_iter(|j| {
                (0..self.image_width).map(move |i| {
                    let mut pixel_color = Color::ZERO;
                    for _sample in 0..self.samples_per_pixel {
                        let r = self.get_ray(i, j);
                        pixel_color += &self.ray_color(&r, self.max_depth, world);
                    }

                    pixel_color * self.pixel_samples_scale
                })
            })
            .collect();

        writeln!(out, "P3\n{} {}\n255", self.image_width, self.image_height)?;

        for pixel in pixels {
            pixel.write_color(out)?;
        }

        pb.finish_with_message("Rendering complete");

        Ok(())
    }

    /// Construct a camera ray originating from the defocus disk and directed
    /// at randomly sampled point around the pixel location i, j.
    fn get_ray(&self, i: i32, j: i32) -> Ray {
        let offset = sample_square();
        let pixel_sample = &self.pixel00_loc
            + ((i as f64 + offset.x()) * &self.pixel_delta_u)
            + ((j as f64 + offset.y()) * &self.pixel_delta_v);

        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center.clone()
        } else {
            self.defocus_disk_sample()
        };

        let ray_direction = pixel_sample - &ray_origin;
        let ray_time = rand::random();

        Ray::new_with_time(ray_origin, ray_direction, ray_time)
    }

    fn ray_color(&self, r: &Ray, depth: i32, world: &HittableList) -> Color {
        // If exceeded ray bounce limit, no more light is gathered
        if depth <= 0 {
            return Color::ZERO;
        }

        if let Some(rec) = world.hit(r, Interval::new(0.001, f64::INFINITY)) {
            if let Some(scatter) = rec.mat.scatter(r, &rec) {
                return scatter.attenuation * self.ray_color(&scatter.scattered, depth - 1, world);
            } else {
                return Color::ZERO;
            }
        }

        let unit_direction = r.direction().unit_vector();
        let a = 0.5 * (unit_direction.y() + 1.0);

        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }

    fn defocus_disk_sample(&self) -> Point3 {
        let p = Vec3::random_in_unit_disk();

        &self.center + (p.x() * &self.defocus_disk_u) + (p.y() * &self.defocus_disk_v)
    }
}

/// Returns the vector to a random point in the [-.5,-.5]-[+.5,+.5] unit square
fn sample_square() -> Vec3 {
    let mut rand = rand::rng();
    Vec3::new(rand.random::<f64>() - 0.5, rand.random::<f64>() - 0.5, 0.0)
}
