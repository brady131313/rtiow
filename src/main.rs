use std::{error::Error, io::BufWriter};

use argh::FromArgs;
use rtiow::{camera::Camera, hittable::HittableList, sphere::Sphere, vec::Point3};

#[derive(FromArgs)]
/// camera/image options
struct Args {
    #[argh(option, short = 'w', default = "400")]
    /// image width
    image_width: i32,
    #[argh(option, short = 's', default = "400")]
    /// samples per pixel for antialiasing
    samples_per_pixel: i32,
    #[argh(option, short = 'd', default = "10")]
    /// max number of ray bounces into scene
    max_depth: i32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut world = HittableList::default();
    world.add(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5));
    world.add(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0));

    let args: Args = argh::from_env();

    let camera = Camera::builder()
        .image_width(args.image_width)
        .aspect_ratio(16.0 / 9.0)
        .samples_per_pixel(args.samples_per_pixel)
        .max_depth(args.max_depth)
        .build();

    let stdout = std::io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    camera
        .render(&world, &mut writer, &mut std::io::stderr())
        .unwrap();

    Ok(())
}
