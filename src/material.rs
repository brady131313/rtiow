use serde::{Deserialize, Serialize};

use crate::{color::Color, hittable::HitRecord, ray::Ray, vec::Vec3};

pub struct ScatterRecord {
    pub attenuation: Color,
    pub scattered: Ray,
}

pub trait Material {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lambertian {
    albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let mut scatter_direction = &rec.normal + Vec3::random_unit_vector();

        // Catch degenerate scatter direction
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal.clone();
        }

        Some(ScatterRecord {
            attenuation: self.albedo.clone(),
            scattered: Ray::new(rec.p.clone(), scatter_direction),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metal {
    albedo: Color,
    fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        Self {
            albedo,
            fuzz: fuzz.min(1.0),
        }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let mut reflected = Vec3::reflect(r_in.direction(), &rec.normal);
        reflected = reflected.unit_vector() + (self.fuzz * Vec3::random_unit_vector());

        let scattered = Ray::new(rec.p.clone(), reflected);
        if scattered.direction().dot(&rec.normal) > 0.0 {
            Some(ScatterRecord {
                attenuation: self.albedo.clone(),
                scattered,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dielectric {
    /// Refractive index in vacuum or air, or the ratio of the material's refractive index
    /// over the refractive index of the enclosing media
    refraction_index: f64,
}

impl Dielectric {
    pub fn new(refraction_index: f64) -> Self {
        Self { refraction_index }
    }

    /// Use Schlick's approximation for reflectance
    fn reflectance(cosine: f64, refraction_index: f64) -> f64 {
        let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let attenuation = Color::new(1.0, 1.0, 1.0);
        let ri = if rec.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = r_in.direction().unit_vector();
        let cos_theta = (-&unit_direction).dot(&rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = ri * sin_theta > 1.0;
        let direction = if cannot_refract || Self::reflectance(cos_theta, ri) > rand::random() {
            Vec3::reflect(&unit_direction, &rec.normal)
        } else {
            Vec3::refract(&unit_direction, &rec.normal, ri)
        };

        let scattered = Ray::new(rec.p.clone(), direction);
        Some(ScatterRecord {
            attenuation,
            scattered,
        })
    }
}
