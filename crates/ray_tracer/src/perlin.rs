use crate::vec::{Point3, Vec3};

pub struct Perlin {
    randvec: [Vec3; Self::POINT_COUNT],
    perm_x: [i32; Self::POINT_COUNT],
    perm_y: [i32; Self::POINT_COUNT],
    perm_z: [i32; Self::POINT_COUNT],
}

impl Default for Perlin {
    fn default() -> Self {
        let mut randvec = [Vec3::ZERO; Self::POINT_COUNT];
        for v in &mut randvec {
            *v = Vec3::random_bounded(-1.0, 1.0).unit_vector();
        }

        let perm_x = Self::perlin_generate_perm();
        let perm_y = Self::perlin_generate_perm();
        let perm_z = Self::perlin_generate_perm();

        Self {
            randvec,
            perm_x,
            perm_y,
            perm_z,
        }
    }
}

impl Perlin {
    const POINT_COUNT: usize = 256;

    pub fn noise(&self, p: &Point3) -> f64 {
        let mut u = p.x() - p.x().floor();
        let mut v = p.y() - p.y().floor();
        let mut w = p.z() - p.z().floor();

        u = u * u * (3.0 - 2.0 * u);
        v = v * v * (3.0 - 2.0 * v);
        w = w * w * (3.0 - 2.0 * w);

        let i = p.x().floor() as i64;
        let j = p.y().floor() as i64;
        let k = p.z().floor() as i64;

        let c = core::array::from_fn(|di| {
            core::array::from_fn(|dj| {
                core::array::from_fn(|dk| {
                    let idx = self.perm_x[((i + di as i64) & 255) as usize]
                        ^ self.perm_y[((j + dj as i64) & 255) as usize]
                        ^ self.perm_z[((k + dk as i64) & 255) as usize];

                    self.randvec[idx as usize].clone()
                })
            })
        });

        Self::perlin_interp(c, u, v, w)
    }

    pub fn turb(&self, p: &Point3, depth: i32) -> f64 {
        let mut accum: f64 = 0.0;
        let mut temp_p = p.clone();
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(&temp_p);
            weight *= 0.5;
            temp_p *= 2.0;
        }

        accum.abs()
    }

    fn perlin_generate_perm() -> [i32; Self::POINT_COUNT] {
        let mut p = [0; Self::POINT_COUNT];
        for (i, p) in p.iter_mut().enumerate() {
            *p = i as i32;
        }

        for i in (1..Self::POINT_COUNT).rev() {
            let target = rand::random_range(0..i);
            p.swap(i, target);
        }

        p
    }

    fn perlin_interp(c: [[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);
        let mut accum = 0.0;

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let weight_v = Vec3::new(u - i as f64, v - j as f64, w - k as f64);
                    accum += (i as f64 * uu + (1 - i) as f64 * (1.0 - uu))
                        * (j as f64 * vv + (1 - j) as f64 * (1.0 - vv))
                        * (k as f64 * ww + (1 - k) as f64 * (1.0 - ww))
                        * c[i][j][k].dot(&weight_v)
                }
            }
        }

        accum
    }
}
