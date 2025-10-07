use std::{error::Error, io::BufWriter, path::PathBuf};

use argh::FromArgs;
use rtiow::{camera::Camera, scene_loader::load_scene};

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
    #[argh(option, default = "90.0")]
    /// vertical field of view
    vfov: f64,
    #[argh(positional)]
    /// the scene file to render
    scene_path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = argh::from_env();

    let world = load_scene(args.scene_path)?;

    let camera = Camera::builder()
        .image_width(args.image_width)
        .aspect_ratio(16.0 / 9.0)
        .samples_per_pixel(args.samples_per_pixel)
        .max_depth(args.max_depth)
        .vfov(args.vfov)
        .build();

    let stdout = std::io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    camera.render(&world, &mut writer).unwrap();

    Ok(())
}
