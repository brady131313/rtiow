use std::{error::Error, io::BufWriter, sync::Arc};

use argh::FromArgs;
use rtiow::{
    camera::Camera,
    color::Color,
    hittable::HittableList,
    material::{Lambertian, Metal},
    sphere::Sphere,
    vec::Point3,
};

#[derive(FromArgs)]
/// camera/image options
struct Args {
    #[argh(option, short = 'w', default = "400")]
    /// image width
    image_width: i32,
    #[argh(option, short = 's', default = "100")]
    /// samples per pixel for antialiasing
    samples_per_pixel: i32,
    #[argh(option, short = 'd', default = "10")]
    /// max number of ray bounces into scene
    max_depth: i32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = argh::from_env();

    let mut world = HittableList::default();

    let material_ground = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let material_center = Arc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    let material_left = Arc::new(Metal::new(Color::new(0.8, 0.8, 0.8), 0.0));
    let material_right = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 1.0));

    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, -100.5, -1.0),
        100.0,
        material_ground,
    )));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 0.0, -1.2),
        0.5,
        material_center,
    )));
    world.add(Arc::new(Sphere::new(
        Point3::new(-1.0, 0.0, -1.0),
        0.5,
        material_left,
    )));
    world.add(Arc::new(Sphere::new(
        Point3::new(1.0, 0.0, -1.0),
        0.5,
        material_right,
    )));

    let camera = Camera::builder()
        .image_width(args.image_width)
        .aspect_ratio(16.0 / 9.0)
        .samples_per_pixel(args.samples_per_pixel)
        .max_depth(args.max_depth)
        .build();

    let stdout = std::io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    camera.render(&world, &mut writer).unwrap();

    Ok(())
}
