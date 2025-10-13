use std::{f64, sync::Arc};

use crate::{
    aabb::AABB,
    hittable::{HitRecord, Hittable},
    interval::Interval,
    material::DynMaterial,
    ray::Ray,
    scene_loader::{ResourceRegistry, ShapeSpec},
    vec::{Point3, Vec3},
};

pub struct Sphere {
    center: Ray,
    radius: f64,
    mat: Arc<DynMaterial>,
    bbox: AABB,
}

impl Sphere {
    pub fn new(static_center: Point3, radius: f64, mat: Arc<DynMaterial>) -> Self {
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

    pub fn new_moving(
        center_1: Point3,
        center_2: Point3,
        radius: f64,
        mat: Arc<DynMaterial>,
    ) -> Self {
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

    /// p: a given point on the sphere of radius one, centered at the origin
    /// u: returned value [0, 1] of angle around the Y axis from X=-1
    /// v: returned value [0, 1] of angle from Y=-1 to Y=+1
    ///
    ///     <1 0 0> yields <0.5 0.5>   <-1 0 0> yields <0.0 0.5>
    ///     <0 1 0> yields <0.5 1.0>   <0 -1 0> yields <0.5 0.0>
    ///     <0 0 1> yields <0.25 0.5>  <0 0 -1> yields <0.75 0.5>
    fn get_sphere_uv(p: &Point3) -> (f64, f64) {
        let theta = f64::acos(-p.y());
        let phi = f64::atan2(-p.z(), p.x()) + f64::consts::PI;

        let u = phi / (2.0 * f64::consts::PI);
        let v = theta / f64::consts::PI;
        (u, v)
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
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

        let (u, v) = Self::get_sphere_uv(&outward_normal);
        rec.u = u;
        rec.v = v;

        Some(rec)
    }

    fn bounding_box(&self) -> &AABB {
        &self.bbox
    }

    fn to_spec(&self, registry: &mut ResourceRegistry) -> ShapeSpec {
        let material_spec = self.mat.to_spec(registry);
        registry.register_material(self.mat.name().to_owned(), material_spec);

        ShapeSpec::Circle {
            radius: self.radius,
            center: self.center.clone(),
            material: self.mat.name().to_owned(),
        }
    }
}
