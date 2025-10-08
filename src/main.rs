use std::{error::Error, io::BufWriter, path::PathBuf, sync::Arc};

use argh::FromArgs;
use rtiow::{
    camera::Camera,
    color::Color,
    hittable::HittableList,
    material::{Dielectric, Lambertian, Metal},
    scene_loader::load_scene,
    sphere::Sphere,
    vec::{Point3, Vec3},
};

#[derive(FromArgs)]
/// camera/image options
struct Args {
    #[argh(option, short = 'r', default = "16.0/9.0")]
    /// aspect ratio
    aspect_ratio: f64,
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
    #[argh(option, default = "Point3::ZERO")]
    /// point camera is looking from
    lookfrom: Point3,
    #[argh(option, default = "Point3::new(0.0, 0.0, -1.0)")]
    /// point camera is looking at
    lookat: Point3,
    #[argh(option, default = "Vec3::new(0.0, 1.0, 0.0)")]
    /// camera relative up direction
    vup: Vec3,
    #[argh(option, default = "0.0")]
    /// variation angle of rays through each pixel
    defocus_angle: f64,
    #[argh(option, default = "10.0")]
    /// distance from camera lookfrom point to plane of perfect focus
    focus_dist: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = argh::from_env();

    // let world = load_scene(args.scene_path)?;
    let mut world = HittableList::default();

    let ground_material = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    )));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat: f64 = rand::random();
            let center = Point3::new(
                a as f64 + 0.9 * rand::random::<f64>(),
                0.2,
                b as f64 + 0.9 * rand::random::<f64>(),
            );

            if (&center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    // diffuse
                    let albedo = Color::random() * Color::random();
                    let sphere_material = Arc::new(Lambertian::new(albedo));
                    world.add(Arc::new(Sphere::new(center.clone(), 0.2, sphere_material)));
                } else if choose_mat < 0.95 {
                    // metal
                    let albedo = Color::random_bounded(0.5, 1.0);
                    let fuzz = rand::random_range(0.0..0.5);
                    let sphere_material = Arc::new(Metal::new(albedo, fuzz));
                    world.add(Arc::new(Sphere::new(center.clone(), 0.2, sphere_material)));
                } else {
                    // glass
                    let sphere_material = Arc::new(Dielectric::new(1.5));
                    world.add(Arc::new(Sphere::new(center.clone(), 0.2, sphere_material)));
                }
            }
        }
    }

    let material_1 = Arc::new(Dielectric::new(1.5));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        material_1,
    )));

    let material_2 = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    world.add(Arc::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        material_2,
    )));

    let material_3 = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
    world.add(Arc::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        material_3,
    )));

    let camera = Camera::builder()
        .image_width(args.image_width)
        .aspect_ratio(args.aspect_ratio)
        .samples_per_pixel(args.samples_per_pixel)
        .max_depth(args.max_depth)
        .vfov(args.vfov)
        .lookfrom(args.lookfrom)
        .lookat(args.lookat)
        .vup(args.vup)
        .defocus_angle(args.defocus_angle)
        .focus_dist(args.focus_dist)
        .build();

    let stdout = std::io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    camera.render(&world, &mut writer).unwrap();

    Ok(())
}
