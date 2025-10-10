use serde::{Deserialize, Serialize};

use crate::{color::Color, hittable::HitRecord, ray::Ray, vec::Vec3};

pub struct ScatterRecord {
    pub attenuation: Color,
    pub scattered: Ray,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Material {
    name: String,
    kind: MaterialKind,
}

impl Material {
    pub fn new(name: impl Into<String>, kind: impl Into<MaterialKind>) -> Self {
        Self {
            name: name.into(),
            kind: kind.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> &MaterialKind {
        &self.kind
    }

    pub fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        match &self.kind {
            MaterialKind::Lambertian(lambertian) => lambertian.scatter(r_in, rec),
            MaterialKind::Metal(metal) => metal.scatter(r_in, rec),
            MaterialKind::Dielectric(dielectric) => dielectric.scatter(r_in, rec),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MaterialKind {
    Lambertian(Lambertian),
    Metal(Metal),
    Dielectric(Dielectric),
}

impl From<Lambertian> for MaterialKind {
    fn from(value: Lambertian) -> Self {
        Self::Lambertian(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Lambertian {
    albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }

    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let mut scatter_direction = &rec.normal + Vec3::random_unit_vector();

        // Catch degenerate scatter direction
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal.clone();
        }

        Some(ScatterRecord {
            attenuation: self.albedo.clone(),
            scattered: Ray::new_with_time(rec.p.clone(), scatter_direction, r_in.time()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metal {
    albedo: Color,
    fuzz: f64,
}

impl From<Metal> for MaterialKind {
    fn from(value: Metal) -> Self {
        Self::Metal(value)
    }
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        Self {
            albedo,
            fuzz: fuzz.min(1.0),
        }
    }

    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let mut reflected = Vec3::reflect(r_in.direction(), &rec.normal);
        reflected = reflected.unit_vector() + (self.fuzz * Vec3::random_unit_vector());

        let scattered = Ray::new_with_time(rec.p.clone(), reflected, r_in.time());
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Dielectric {
    /// Refractive index in vacuum or air, or the ratio of the material's refractive index
    /// over the refractive index of the enclosing media
    refraction_index: f64,
}

impl From<Dielectric> for MaterialKind {
    fn from(value: Dielectric) -> Self {
        Self::Dielectric(value)
    }
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

        let scattered = Ray::new_with_time(rec.p.clone(), direction, r_in.time());
        Some(ScatterRecord {
            attenuation,
            scattered,
        })
    }
}
