use std::{collections::HashMap, fs::File, io::BufReader, path::Path, sync::Arc};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{
    hittable::{Hittable, HittableList},
    material::{Material, MaterialKind},
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

#[derive(Deserialize, Serialize)]
pub struct SceneFile {
    materials: HashMap<String, MaterialKind>,
    scene: Vec<Shapes>,
}

impl From<HittableList> for SceneFile {
    fn from(value: HittableList) -> Self {
        let mut materials = HashMap::new();
        let mut scene = Vec::new();

        // for obj in value.objects() {
        //     match obj.as_ref() {
        //         Hittable::Sphere(sphere) => {
        //             // comment
        //             materials.insert(sphere.mat.name().to_owned(), sphere.mat.kind().clone());
        //             scene.push(Shapes::Circle {
        //                 radius: sphere.radius,
        //                 center: sphere.center.clone(),
        //                 material: sphere.mat.name().to_owned(),
        //             });
        //         }
        //     }
        // }
        todo!();

        Self { materials, scene }
    }
}

pub fn load_scene<P: AsRef<Path>>(path: P) -> anyhow::Result<HittableList> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let scene_file: SceneFile =
        serde_json::from_reader(reader).context("Failed to load scene file")?;

    let mut materials = HashMap::new();
    for (name, material) in scene_file.materials {
        materials.insert(name.clone(), Arc::new(Material::new(name, material)));
    }

    let mut world = HittableList::default();
    for raw_shape in scene_file.scene {
        let Some(material) = materials.get(raw_shape.material()) else {
            anyhow::bail!("unknown material name: {}", raw_shape.material())
        };

        let shape = match raw_shape {
            Shapes::Circle { radius, center, .. } => {
                Hittable::Sphere(Sphere::new(center, radius, material.clone()))
            }
        };

        world.add(Arc::new(shape));
    }

    Ok(world)
}
