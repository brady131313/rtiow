use std::{cmp::Ordering, sync::Arc};

use crate::{
    aabb::AABB,
    hittable::{DynHittable, HitRecord, Hittable, HittableList},
    interval::Interval,
    ray::Ray,
    scene_loader::{ResourceRegistry, ShapeSpec},
    vec::Axis,
};

pub struct BVHNode {
    left: Arc<DynHittable>,
    right: Arc<DynHittable>,
    bbox: AABB,
}

impl BVHNode {
    pub fn new(mut list: HittableList) -> Self {
        Self::from_slice(list.objects_mut())
    }

    pub fn from_slice(objects: &mut [Arc<DynHittable>]) -> Self {
        let mut bbox = AABB::EMPTY;
        for object in objects.iter() {
            bbox = AABB::from_boxes(&bbox, object.bounding_box());
        }

        let (left, right) = if objects.len() == 1 {
            (objects[0].clone(), objects[0].clone())
        } else if objects.len() == 2 {
            (objects[0].clone(), objects[1].clone())
        } else {
            let axis = bbox.longest_axis();
            objects.sort_by(|a, b| box_compare(a, b, axis));

            let (left_objs, right_objs) = objects.split_at_mut(objects.len() / 2);
            let left: Arc<DynHittable> = Arc::new(Self::from_slice(left_objs));
            let right: Arc<DynHittable> = Arc::new(Self::from_slice(right_objs));
            (left, right)
        };

        Self { left, right, bbox }
    }
}

impl Hittable for BVHNode {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        if !self.bbox.hit(r, ray_t.clone()) {
            return None;
        }

        let hit_left = self.left.hit(r, ray_t.clone());

        let right_endpoint = if let Some(rec) = &hit_left {
            rec.t
        } else {
            ray_t.max
        };

        let hit_right = self.right.hit(r, Interval::new(ray_t.min, right_endpoint));

        match (hit_left, hit_right) {
            (Some(lhs), Some(rhs)) => Some(if rhs.t < lhs.t { rhs } else { lhs }),
            (Some(lhs), None) => Some(lhs),
            (None, Some(rhs)) => Some(rhs),
            (None, None) => None,
        }
    }

    fn bounding_box(&self) -> &AABB {
        &self.bbox
    }

    fn to_spec(&self, registry: &mut ResourceRegistry) -> ShapeSpec {
        ShapeSpec::BVH {
            left: Box::new(self.left.to_spec(registry)),
            right: Box::new(self.right.to_spec(registry)),
        }
    }
}

fn box_compare<A: Hittable, B: Hittable>(a: &A, b: &B, axis: Axis) -> Ordering {
    let a_axis_interval = a.bounding_box().axis_interval(axis);
    let b_axis_interval = b.bounding_box().axis_interval(axis);
    a_axis_interval.min.total_cmp(&b_axis_interval.min)
}
