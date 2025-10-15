use crate::{
    interval::Interval,
    ray::Ray,
    vec::{Axis, Point3},
};

#[derive(Default)]
pub struct AABB {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}

impl AABB {
    pub const EMPTY: Self = Self::new(Interval::EMPTY, Interval::EMPTY, Interval::EMPTY);
    pub const UNIVERSE: Self =
        Self::new(Interval::UNIVERSE, Interval::UNIVERSE, Interval::UNIVERSE);

    pub const fn new(x: Interval, y: Interval, z: Interval) -> Self {
        Self {
            x: pad_to_minimums(x),
            y: pad_to_minimums(y),
            z: pad_to_minimums(z),
        }
    }

    /// Treat the two points a and b as extrema for the bounding box
    pub fn from_points(a: Point3, b: Point3) -> Self {
        let x = Interval::new(a.x().min(b.x()), a.x().max(b.x()));
        let y = Interval::new(a.y().min(b.y()), a.y().max(b.y()));
        let z = Interval::new(a.z().min(b.z()), a.z().max(b.z()));

        Self {
            x: pad_to_minimums(x),
            y: pad_to_minimums(y),
            z: pad_to_minimums(z),
        }
    }

    pub fn from_boxes(box0: &AABB, box1: &AABB) -> Self {
        Self {
            x: Interval::from_intervals(&box0.x, &box1.x),
            y: Interval::from_intervals(&box0.y, &box1.y),
            z: Interval::from_intervals(&box0.z, &box1.z),
        }
    }

    pub fn axis_interval(&self, axis: Axis) -> &Interval {
        match axis {
            Axis::X => &self.x,
            Axis::Y => &self.y,
            Axis::Z => &self.z,
        }
    }

    pub fn hit(&self, r: &Ray, mut ray_t: Interval) -> bool {
        let ray_orig = r.origin();
        let ray_dir = r.direction();

        for axis in Axis::iter() {
            let ax = self.axis_interval(axis);
            let adinv = 1.0 / ray_dir[axis];

            let t0 = (ax.min - ray_orig[axis]) * adinv;
            let t1 = (ax.max - ray_orig[axis]) * adinv;

            if t0 < t1 {
                if t0 > ray_t.min {
                    ray_t.min = t0;
                }
                if t1 < ray_t.max {
                    ray_t.max = t1;
                }
            } else {
                if t1 > ray_t.min {
                    ray_t.min = t1;
                }
                if t0 < ray_t.max {
                    ray_t.max = t0;
                }
            }

            if ray_t.max <= ray_t.min {
                return false;
            }
        }

        true
    }

    pub fn longest_axis(&self) -> Axis {
        if self.x.size() > self.y.size() {
            if self.x.size() > self.z.size() {
                Axis::X
            } else {
                Axis::Z
            }
        } else if self.y.size() > self.z.size() {
            Axis::Y
        } else {
            Axis::Z
        }
    }
}

const fn pad_to_minimums(interval: Interval) -> Interval {
    let delta = 0.0001;
    if interval.size() < delta {
        interval.expand(delta)
    } else {
        interval
    }
}
