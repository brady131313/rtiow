use std::io::Write;

use rtiow::{
    color::Color,
    ray::Ray,
    vec::{Point3, Vec3},
};

fn main() {
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 1920;

    // Calculate image height, bounded below by 1
    let image_height = ((image_width as f64 / aspect_ratio) as i32).max(1);

    // Camera
    let focal_length = 1.0;
    let viewport_height = 2.0;
    // Don't use aspect_ratio because we need real valued aspect ratio
    let viewport_width = viewport_height * (image_width as f64 / image_height as f64);
    let camera_center = Point3::ZERO;

    // Calculate vectors across horizontal and down vertical viewport edges
    let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
    let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);

    // Calculate horizontal and vertical delta vectors from pixel to pixel
    let pixel_delta_u = &viewport_u / image_width as f64;
    let pixel_delta_v = &viewport_v / image_height as f64;

    // Calculate the location of the upper left pixel
    let viewport_upper_left =
        &camera_center - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
    let pixel00_loc = viewport_upper_left + 0.5 * (&pixel_delta_u + &pixel_delta_v);

    println!("P3\n{image_width} {image_height}\n255");
    for j in 0..image_height {
        eprint!("\rScanlines remaining: {} ", image_height - j);
        std::io::stderr().flush().unwrap();

        for i in 0..image_width {
            let pixel_center =
                &pixel00_loc + (i as f64 * &pixel_delta_u) + (j as f64 * &pixel_delta_v);
            let ray_direction = pixel_center - &camera_center;

            let r = Ray::new(camera_center.clone(), ray_direction);
            let pixel_color = ray_color(&r);
            println!("{pixel_color}");
        }
    }

    eprintln!("\rDone.                         ")
}

fn hit_sphere(center: &Point3, radius: f64, r: &Ray) -> bool {
    let oc = center - r.origin();
    let a = r.direction().dot(r.direction());
    let b = -2.0 * r.direction().dot(&oc);
    let c = oc.dot(&oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;

    discriminant >= 0.0
}

fn ray_color(r: &Ray) -> Color {
    if hit_sphere(&Point3::new(0.0, 0.0, 1.0), 0.5, r) {
        return Color::new(1.0, 0.0, 0.0);
    }

    let unit_direction = r.direction().unit_vector();
    let a = 0.5 * (unit_direction.y() + 1.0);
    let raw_color = (1.0 - a) * Vec3::new(1.0, 1.0, 1.0) + a * Vec3::new(0.5, 0.7, 1.0);
    Color::from(raw_color)
}
