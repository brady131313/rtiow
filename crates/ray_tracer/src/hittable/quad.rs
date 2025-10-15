use std::sync::Arc;

use crate::{
    aabb::AABB,
    hittable::{HitRecord, Hittable},
    interval::Interval,
    material::DynMaterial,
    ray::Ray,
    scene_loader::{ResourceRegistry, ShapeSpec},
    vec::{Point3, Vec3},
};

pub struct Quad {
    q: Point3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    mat: Arc<DynMaterial>,
    bbox: AABB,
    normal: Vec3,
    d: f64,
}

impl Quad {
    pub fn new(q: Point3, u: Vec3, v: Vec3, mat: Arc<DynMaterial>) -> Self {
        let bbox_diagonal_1 = AABB::from_points(q.clone(), &q + (&u + &v));
        let bbox_diagonal_2 = AABB::from_points(&q + &u, &q + &v);
        let bbox = AABB::from_boxes(&bbox_diagonal_1, &bbox_diagonal_2);

        let n = u.cross(&v);
        let normal = n.unit_vector();
        let d = normal.dot(&q);
        let w = &n / n.dot(&n);

        Self {
            q,
            u,
            v,
            w,
            mat,
            bbox,
            normal,
            d,
        }
    }

    fn is_interior(a: f64, b: f64) -> Option<(f64, f64)> {
        let unit_interval = Interval::new(0.0, 1.0);

        // Given the hit point in plane coordinates, return None if it is outside
        // the primitive, otherwise return the UV coordinates

        if !unit_interval.contains(a) || !unit_interval.contains(b) {
            return None;
        }

        Some((a, b))
    }
}

impl Hittable for Quad {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let denom = self.normal.dot(r.direction());

        // no hit if the ray is parallel to the plane
        if denom.abs() < 1e-8 {
            return None;
        }

        // no hit if the hit point parameter t is outside the ray interval
        let t = (self.d - self.normal.dot(r.origin())) / denom;
        if !ray_t.contains(t) {
            return None;
        }

        // Determine if the hit point lies within the planar shape using its plane coordinates
        let intersection = r.at(t);
        let planar_hitpt_vector = &intersection - &self.q;
        let alpha = self.w.dot(&planar_hitpt_vector.cross(&self.v));
        let beta = self.w.dot(&self.u.cross(&planar_hitpt_vector));

        let mut rec = HitRecord::new(intersection, self.normal.clone(), self.mat.clone(), t);
        let (u, v) = Self::is_interior(alpha, beta)?;
        rec.u = u;
        rec.v = v;

        rec.set_face_normal(r, &self.normal);

        Some(rec)
    }

    fn bounding_box(&self) -> &AABB {
        &self.bbox
    }

    fn to_spec(&self, registry: &mut ResourceRegistry) -> ShapeSpec {
        let material_spec = self.mat.to_spec(registry);
        registry.register_material(self.mat.name().to_owned(), material_spec);

        ShapeSpec::Quad {
            q: self.q.clone(),
            u: self.u.clone(),
            v: self.v.clone(),
            material: self.mat.name().to_owned(),
        }
    }
}
