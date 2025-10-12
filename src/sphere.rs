use std::sync::Arc;

use crate::{
    aabb::AABB,
    hittable::HitRecord,
    interval::Interval,
    material::Material,
    ray::Ray,
    vec::{Point3, Vec3},
};

pub struct Sphere {
    center: Ray,
    radius: f64,
    mat: Arc<Material>,
    bbox: AABB,
}

impl Sphere {
    pub fn new(static_center: Point3, radius: f64, mat: Arc<Material>) -> Self {
        let radius = radius.max(0.0);

        let rvec = Vec3::new(radius, radius, radius);
        let bbox = AABB::from_points(&static_center - &rvec, &static_center + &rvec);

        Self {
            center: Ray::new(static_center, Vec3::ZERO),
            radius,
            mat,
            bbox,
        }
    }

    pub fn new_moving(center_1: Point3, center_2: Point3, radius: f64, mat: Arc<Material>) -> Self {
        let radius = radius.max(0.0);
        let center = Ray::new(center_1.clone(), center_2 - center_1);

        let rvec = Vec3::new(radius, radius, radius);
        let box1 = AABB::from_points(center.at(0.0) - &rvec, &center.at(0.0) + &rvec);
        let box2 = AABB::from_points(center.at(1.0) - &rvec, center.at(1.0) + rvec);
        let bbox = AABB::from_boxes(&box1, &box2);

        Self {
            center,
            radius,
            mat,
            bbox,
        }
    }

    pub fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let current_center = self.center.at(r.time());
        let oc = &current_center - r.origin();
        let a = r.direction().length_squared();
        let h = r.direction().dot(&oc);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant.is_sign_negative() {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in the acceptable range
        let mut root = (h - sqrtd) / a;
        if !ray_t.surrounds(root) {
            root = (h + sqrtd) / a;
            if !ray_t.surrounds(root) {
                return None;
            }
        }

        let t = root;
        let p = r.at(t);
        let outward_normal = (&p - &current_center) / self.radius;
        let mut rec = HitRecord::new(p, outward_normal.clone(), self.mat.clone(), t);
        rec.set_face_normal(r, &outward_normal);

        Some(rec)
    }

    pub fn bounding_box(&self) -> &AABB {
        &self.bbox
    }
}
