use std::sync::Arc;

use crate::{
    aabb::AABB,
    interval::Interval,
    material::Material,
    ray::Ray,
    sphere::Sphere,
    vec::{Point3, Vec3},
};

pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub mat: Arc<Material>,
    pub t: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn new(p: Point3, normal: Vec3, mat: Arc<Material>, t: f64) -> Self {
        Self {
            p,
            normal,
            mat,
            t,
            front_face: false,
        }
    }

    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: &Vec3) {
        // Ray is outside sphere. negative dot product implies
        // vectors are facing opposite directions and normal of
        // geometry should always be facing outward
        self.front_face = r.direction().dot(outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal.clone()
        } else {
            -outward_normal
        }
    }
}

pub enum Hittable {
    Sphere(Sphere),
}

impl Hittable {
    pub fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        match self {
            Self::Sphere(sphere) => sphere.hit(r, ray_t),
        }
    }

    pub fn bounding_box(&self) -> &AABB {
        match self {
            Self::Sphere(sphere) => sphere.bounding_box(),
        }
    }
}

#[derive(Default)]
pub struct HittableList {
    objects: Vec<Arc<Hittable>>,
    bbox: AABB,
}

impl HittableList {
    pub fn add(&mut self, object: Arc<Hittable>) {
        self.bbox = AABB::from_boxes(&self.bbox, object.bounding_box());
        self.objects.push(object);
    }

    pub fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let mut hit_anything = None;
        let mut closest_so_far = ray_t.max;

        for object in &self.objects {
            if let Some(rec) = object.hit(r, Interval::new(ray_t.min, closest_so_far)) {
                closest_so_far = rec.t;
                hit_anything = Some(rec);
            }
        }

        hit_anything
    }

    pub fn objects(&self) -> &[Arc<Hittable>] {
        &self.objects
    }

    pub fn bounding_box(&self) -> &AABB {
        &self.bbox
    }
}
