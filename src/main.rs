use std::{error::Error, io::BufWriter};

use rtiow::{camera::Camera, hittable::HittableList, sphere::Sphere, vec::Point3};

fn main() -> Result<(), Box<dyn Error>> {
    let mut world = HittableList::default();
    world.add(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5));
    world.add(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0));

    let camera = Camera::builder()
        .image_width(400)
        .aspect_ratio(16.0 / 9.0)
        .samples_per_pixel(100)
        .build();

    let stdout = std::io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    camera
        .render(&world, &mut writer, &mut std::io::stderr())
        .unwrap();

    Ok(())
}
