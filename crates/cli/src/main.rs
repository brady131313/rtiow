use std::{
    error::Error,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};

use anyhow::Context;
use argh::FromArgs;
use indicatif::{ProgressBar, ProgressStyle};
use ray_tracer::{
    camera::{Camera, PPMRenderWriter, RenderProgressTracker},
    color::Color,
    hittable::{HittableList, bvh::BVHNode, quad::Quad, sphere::Sphere},
    material::{Dielectric, Lambertian, Metal},
    scene_loader::SceneFile,
    texture::{CheckerTexture, ImageTexture, NoiseTexture},
    vec::{Point3, Vec3},
};

#[derive(FromArgs)]
/// ray tracing in one weekend command
struct Args {
    #[argh(subcommand)]
    command: SubCommand,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum SubCommand {
    Render(RenderSceneArgs),
    Dump(DumpSceneArgs),
}

#[derive(FromArgs)]
/// camera/image options
#[argh(subcommand, name = "render")]
struct RenderSceneArgs {
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
    #[argh(
        option,
        short = 'o',
        default = "PathBuf::from_str(\"image.ppm\").unwrap()"
    )]
    /// output file
    output_path: PathBuf,
    #[argh(positional)]
    /// the scene file to render
    scene_path: PathBuf,
}

#[derive(FromArgs)]
/// dump a hard coded scene
#[argh(subcommand, name = "dump")]
struct DumpSceneArgs {
    #[argh(positional)]
    /// scene id to dump
    scene: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = argh::from_env();

    match args.command {
        SubCommand::Render(args) => {
            let file = File::open(args.scene_path)?;
            let reader = BufReader::new(file);
            let scene: SceneFile =
                serde_json::from_reader(reader).context("Failed to load scene file")?;

            let world = scene.into_list()?;

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

            let output = File::create(args.output_path)?;
            let writer = BufWriter::new(output);
            let mut writer = PPMRenderWriter::new(writer);

            let pb = ProgressBar::no_length();
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                .unwrap()
            );

            let mut pb = IndicatifProgressTracker(pb);

            camera.render(&world, &mut writer, &mut pb).unwrap();

            pb.0.finish_with_message("Rendering complete");
        }
        SubCommand::Dump(args) => {
            let world = match args.scene.as_str() {
                "cover" => Ok(book_cover()),
                "checkered_spheres" => Ok(checkered_spheres()),
                "earth" => earth(),
                "perlin_spheres" => Ok(perlin_spheres()),
                "quads" => quads(),
                _ => Err(anyhow::anyhow!("invalid scene id: '{}'", args.scene)),
            }?;

            let scene: SceneFile = world.into();

            let stdout = std::io::stdout();
            let writer = BufWriter::new(stdout.lock());

            serde_json::to_writer_pretty(writer, &scene)?;
        }
    }

    Ok(())
}

struct IndicatifProgressTracker(ProgressBar);

impl RenderProgressTracker for IndicatifProgressTracker {
    fn init(&self, total: usize) {
        self.0.set_length(total as u64);
    }

    fn tick(&self, _current: usize) {
        self.0.inc(1);
    }
}

fn book_cover() -> HittableList {
    let mut world = HittableList::default();
    let checker = Arc::new(CheckerTexture::from_color(
        "checker",
        0.32,
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));

    let ground_material = Arc::new(Lambertian::from_texture(checker));
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
                    let name = format!("diffuse_{a}_{b}");
                    let albedo = Color::random() * Color::random();
                    let sphere_material = Arc::new(Lambertian::new(name, albedo));
                    let center_2 = &center + Vec3::new(0.0, rand::random_range(0.0..0.5), 0.0);
                    world.add(Arc::new(Sphere::new_moving(
                        center.clone(),
                        center_2,
                        0.2,
                        sphere_material,
                    )));
                } else if choose_mat < 0.95 {
                    // metal
                    let name = format!("metal_{a}_{b}");
                    let albedo = Color::random_bounded(0.5, 1.0);
                    let fuzz = rand::random_range(0.0..0.5);
                    let sphere_material = Arc::new(Metal::new(name, albedo, fuzz));
                    world.add(Arc::new(Sphere::new(center.clone(), 0.2, sphere_material)));
                } else {
                    // glass
                    let name = format!("glass_{a}_{b}");
                    let sphere_material = Arc::new(Dielectric::new(name, 1.5));
                    world.add(Arc::new(Sphere::new(center.clone(), 0.2, sphere_material)));
                }
            }
        }
    }

    let material_1 = Arc::new(Dielectric::new("material_1", 1.5));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        material_1,
    )));

    let material_2 = Arc::new(Lambertian::new("material_2", Color::new(0.4, 0.2, 0.1)));
    world.add(Arc::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        material_2,
    )));

    let material_3 = Arc::new(Metal::new("material_3", Color::new(0.7, 0.6, 0.5), 0.0));
    world.add(Arc::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        material_3,
    )));

    let mut bvh_world = HittableList::default();
    bvh_world.add(Arc::new(BVHNode::new(world)));
    bvh_world
}

fn checkered_spheres() -> HittableList {
    let mut world = HittableList::default();

    let checker = Arc::new(CheckerTexture::from_color(
        "checker",
        0.32,
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));

    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, -10.0, 0.0),
        10.0,
        Arc::new(Lambertian::from_texture(checker.clone())),
    )));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 10.0, 0.0),
        10.0,
        Arc::new(Lambertian::from_texture(checker.clone())),
    )));

    world
}

fn earth() -> anyhow::Result<HittableList> {
    let earth_texture = Arc::new(ImageTexture::new("textures/earthmap.jpg")?);
    let earth_surface = Arc::new(Lambertian::from_texture(earth_texture));
    let globe = Arc::new(Sphere::new(Point3::new(0.0, 0.0, 0.0), 2.0, earth_surface));

    let mut world = HittableList::default();
    world.add(globe);

    Ok(world)
}

fn perlin_spheres() -> HittableList {
    let pertext = Arc::new(NoiseTexture::new(4.0));
    let pertext_mat = Arc::new(Lambertian::from_texture(pertext));
    let mut world = HittableList::default();

    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        pertext_mat.clone(),
    )));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 2.0, 0.0),
        2.0,
        pertext_mat.clone(),
    )));

    world
}

fn quads() -> anyhow::Result<HittableList> {
    let mut world = HittableList::default();

    let earth_texture = Arc::new(ImageTexture::new("textures/earthmap.jpg")?);
    let earth_surface = Arc::new(Lambertian::from_texture(earth_texture));

    let left_red = Arc::new(Lambertian::new("left_red", Color::new(1.0, 0.2, 0.2)));
    let back_green = Arc::new(Lambertian::new("back_green", Color::new(0.2, 1.0, 0.2)));
    let right_blue = Arc::new(Lambertian::new("right_blue", Color::new(0.2, 0.2, 1.0)));
    let upper_orange = Arc::new(Lambertian::new("upper_orange", Color::new(1.0, 0.5, 0.0)));
    let lower_teal = Arc::new(Lambertian::new("lower_teal", Color::new(0.2, 0.8, 0.8)));

    world.add(Arc::new(Quad::new(
        Point3::new(-3.0, -2.0, 5.0),
        Vec3::new(0.0, 0.0, -4.0),
        Vec3::new(0.0, 4.0, 0.0),
        left_red.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(-2.0, -2.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 4.0, 0.0),
        earth_surface.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(3.0, -2.0, 1.0),
        Vec3::new(0.0, 0.0, 4.0),
        Vec3::new(0.0, 4.0, 0.0),
        right_blue.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(-2.0, 3.0, 1.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 4.0),
        upper_orange.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(-2.0, -3.0, 5.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -4.0),
        lower_teal.clone(),
    )));

    Ok(world)
}
