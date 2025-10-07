use std::{collections::HashMap, fs::File, io::BufReader, path::Path, sync::Arc};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{
    hittable::HittableList,
    material::{Dielectric, Lambertian, Material, Metal},
    sphere::Sphere,
    vec::Point3,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum Shapes {
    Circle {
        radius: f64,
        center: Point3,
        material: String,
    },
}

impl Shapes {
    pub fn material(&self) -> &str {
        match self {
            Self::Circle { material, .. } => material,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Materials {
    Lambertian(Lambertian),
    Metal(Metal),
    Dielectric(Dielectric),
}

impl Materials {
    pub fn into_dyn(self) -> Arc<dyn Material + Send + Sync> {
        match self {
            Self::Lambertian(lambertian) => Arc::new(lambertian),
            Self::Metal(metal) => Arc::new(metal),
            Self::Dielectric(dielectric) => Arc::new(dielectric),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct SceneFile {
    materials: HashMap<String, Materials>,
    scene: Vec<Shapes>,
}

pub fn load_scene<P: AsRef<Path>>(path: P) -> anyhow::Result<HittableList> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let scene_file: SceneFile =
        serde_json::from_reader(reader).context("Failed to load scene file")?;

    let mut materials = HashMap::new();
    for (name, material) in scene_file.materials {
        materials.insert(name, material.into_dyn());
    }

    let mut world = HittableList::default();
    for raw_shape in scene_file.scene {
        let Some(material) = materials.get(raw_shape.material()) else {
            anyhow::bail!("unknown material name: {}", raw_shape.material())
        };

        let shape = match raw_shape {
            Shapes::Circle { radius, center, .. } => Sphere::new(center, radius, material.clone()),
        };

        world.add(Arc::new(shape));
    }

    Ok(world)
}
