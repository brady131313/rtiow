use std::sync::Arc;

use crate::{
    aabb::AABB,
    interval::Interval,
    material::DynMaterial,
    ray::Ray,
    scene_loader::{ResourceRegistry, ShapeSpec},
    vec::{Point3, Vec3},
};

pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub mat: Arc<DynMaterial>,
    pub t: f64,
    pub u: f64,
    pub v: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn new(p: Point3, normal: Vec3, mat: Arc<DynMaterial>, t: f64) -> Self {
        Self {
            p,
            normal,
            mat,
            t,
            u: 0.0,
            v: 0.0,
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

pub trait Hittable {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord>;

    fn bounding_box(&self) -> &AABB;

    fn to_spec(&self, registry: &mut ResourceRegistry) -> ShapeSpec;
}

pub type DynHittable = dyn Hittable + Send + Sync;

impl Hittable for Arc<DynHittable> {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        (**self).hit(r, ray_t)
    }

    fn bounding_box(&self) -> &AABB {
        (**self).bounding_box()
    }

    fn to_spec(&self, registry: &mut ResourceRegistry) -> ShapeSpec {
        (**self).to_spec(registry)
    }
}

#[derive(Default)]
pub struct HittableList {
    objects: Vec<Arc<DynHittable>>,
    bbox: AABB,
}

impl HittableList {
    pub fn add(&mut self, object: Arc<DynHittable>) {
        self.bbox = AABB::from_boxes(&self.bbox, object.bounding_box());
        self.objects.push(object);
    }

    pub fn objects(&self) -> &[Arc<DynHittable>] {
        &self.objects
    }

    pub fn objects_mut(&mut self) -> &mut [Arc<DynHittable>] {
        &mut self.objects
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
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

    fn bounding_box(&self) -> &AABB {
        &self.bbox
    }

    fn to_spec(&self, registry: &mut ResourceRegistry) -> ShapeSpec {
        let mut specs = Vec::new();
        for obj in self.objects() {
            specs.push(obj.to_spec(registry));
        }

        ShapeSpec::List(specs)
    }
}
