use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{
    bvh::BVHNode,
    color::Color,
    hittable::{DynHittable, HittableList},
    material::{Dielectric, DynMaterial, Lambertian, Metal},
    ray::Ray,
    sphere::Sphere,
    texture::{CheckerTexture, DynTexture, ImageTexture, SolidColor},
};

type MaterialKey = String;
type TextureKey = String;

#[derive(Debug, Serialize, Deserialize)]
pub enum ShapeSpec {
    Circle {
        radius: f64,
        center: Ray,
        material: MaterialKey,
    },
    List(Vec<ShapeSpec>),
    BVH {
        left: Box<ShapeSpec>,
        right: Box<ShapeSpec>,
    },
}

impl ShapeSpec {
    fn build(self, materials: &HashMap<String, Arc<DynMaterial>>) -> Arc<DynHittable> {
        match self {
            Self::Circle {
                radius,
                center,
                material,
            } => {
                let material = materials[&material].clone();
                Arc::new(Sphere::new_moving(
                    center.origin().clone(),
                    center.origin() + center.direction(),
                    radius,
                    material,
                ))
            }
            Self::List(shape_specs) => {
                let mut world = HittableList::default();
                for spec in shape_specs {
                    world.add(spec.build(materials));
                }

                Arc::new(world)
            }
            Self::BVH { left, right } => {
                let left = left.build(materials);
                let right = right.build(materials);

                Arc::new(BVHNode::from_slice(&mut [left, right]))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TextureSpec {
    SolidColor {
        albedo: Color,
    },
    Checker {
        scale: f64,
        even: TextureKey,
        odd: TextureKey,
    },
    Image {
        path: PathBuf,
    },
}

impl TextureSpec {
    fn build(
        self,
        name: &str,
        textures: &HashMap<String, Arc<DynTexture>>,
    ) -> anyhow::Result<Arc<DynTexture>> {
        match self {
            Self::SolidColor { albedo } => Ok(Arc::new(SolidColor::new(name, albedo))),
            Self::Checker { scale, even, odd } => {
                let even = textures[&even].clone();
                let odd = textures[&odd].clone();
                Ok(Arc::new(CheckerTexture::new(scale, even, odd)))
            }
            Self::Image { path } => Ok(Arc::new(ImageTexture::new(&path)?)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MaterialSpec {
    Lambertian { texture: TextureKey },
    Metal { albedo: Color, fuzz: f64 },
    Dielectric { refraction_index: f64 },
}

impl MaterialSpec {
    fn build(self, name: &str, textures: &HashMap<String, Arc<DynTexture>>) -> Arc<DynMaterial> {
        match self {
            Self::Lambertian { texture } => {
                let texture = textures[&texture].clone();
                Arc::new(Lambertian::from_texture(texture))
            }
            Self::Metal { albedo, fuzz } => Arc::new(Metal::new(name, albedo, fuzz)),
            Self::Dielectric { refraction_index } => {
                Arc::new(Dielectric::new(name, refraction_index))
            }
        }
    }
}

#[derive(Default)]
pub struct ResourceRegistry {
    materials: Vec<(String, MaterialSpec)>,
    textures: Vec<(String, TextureSpec)>,
}

impl ResourceRegistry {
    pub fn register_material(&mut self, name: String, spec: MaterialSpec) {
        if self.materials.iter().find(|(n, _)| &name == n).is_some() {
            return;
        }

        self.materials.push((name, spec));
    }

    pub fn register_texture(&mut self, name: String, spec: TextureSpec) {
        if self.textures.iter().find(|(n, _)| &name == n).is_some() {
            return;
        }

        self.textures.push((name, spec));
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SceneFile {
    textures: Vec<(String, TextureSpec)>,
    materials: Vec<(String, MaterialSpec)>,
    shapes: Vec<ShapeSpec>,
}

impl From<HittableList> for SceneFile {
    fn from(value: HittableList) -> Self {
        let mut registry = ResourceRegistry::default();
        let mut shapes = Vec::new();

        for obj in value.objects() {
            let shape_spec = obj.to_spec(&mut registry);
            shapes.push(shape_spec);
        }

        Self {
            materials: registry.materials,
            textures: registry.textures,
            shapes,
        }
    }
}

pub fn load_scene<P: AsRef<Path>>(path: P) -> anyhow::Result<HittableList> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let scene_file: SceneFile =
        serde_json::from_reader(reader).context("Failed to load scene file")?;

    let mut textures: HashMap<String, Arc<DynTexture>> = HashMap::new();
    for (name, spec) in scene_file.textures {
        let texture = spec.build(&name, &textures)?;
        textures.insert(name, texture);
    }

    let mut materials: HashMap<String, Arc<DynMaterial>> = HashMap::new();
    for (name, spec) in scene_file.materials {
        let material = spec.build(&name, &textures);
        materials.insert(name, material);
    }

    let mut world = HittableList::default();
    for shape_spec in scene_file.shapes {
        let hittable = shape_spec.build(&materials);
        world.add(hittable);
    }

    Ok(world)
}
