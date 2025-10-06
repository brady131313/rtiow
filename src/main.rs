use rtiow::{camera::Camera, hittable::HittableList, sphere::Sphere, vec::Point3};

fn main() {
    let mut world = HittableList::default();
    world.add(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5));
    world.add(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0));

    let camera = Camera::new(16.0 / 9.0, 1920);
    camera.render(&world);
}
