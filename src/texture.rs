use std::sync::Arc;

use crate::{
    color::Color,
    scene_loader::{ResourceRegistry, TextureSpec},
    vec::Point3,
};

pub trait Texture {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color;

    fn to_spec(&self, registry: &mut ResourceRegistry) -> TextureSpec;

    fn name(&self) -> &str;
}

pub type DynTexture = dyn Texture + Send + Sync;

pub struct SolidColor {
    name: String,
    albedo: Color,
}

impl SolidColor {
    pub fn new(name: impl Into<String>, albedo: Color) -> Self {
        Self {
            name: name.into(),
            albedo,
        }
    }

    pub fn from_rgb(name: impl Into<String>, red: f64, green: f64, blue: f64) -> Self {
        Self::new(name, Color::new(red, green, blue))
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: &Point3) -> Color {
        self.albedo.clone()
    }

    fn to_spec(&self, _registry: &mut ResourceRegistry) -> TextureSpec {
        TextureSpec::SolidColor {
            albedo: self.albedo.clone(),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct CheckerTexture {
    name: String,
    inv_scale: f64,
    even: Arc<DynTexture>,
    odd: Arc<DynTexture>,
}

impl CheckerTexture {
    pub fn new(scale: f64, even: Arc<DynTexture>, odd: Arc<DynTexture>) -> Self {
        Self {
            name: format!("checker_{}_{}", even.name(), odd.name()),
            inv_scale: 1.0 / scale,
            even,
            odd,
        }
    }

    pub fn from_color(name: impl Into<String>, scale: f64, even: Color, odd: Color) -> Self {
        let name = name.into();
        let even_name = format!("checker_{name}_even");
        let odd_name = format!("checker_{name}_odd");
        Self {
            name,
            inv_scale: 1.0 / scale,
            even: Arc::new(SolidColor::new(even_name, even)),
            odd: Arc::new(SolidColor::new(odd_name, odd)),
        }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color {
        let x_int = (self.inv_scale * p.x()).floor() as i32;
        let y_int = (self.inv_scale * p.y()).floor() as i32;
        let z_int = (self.inv_scale * p.z()).floor() as i32;

        let is_even = (x_int + y_int + z_int) % 2 == 0;
        if is_even {
            self.even.value(u, v, p)
        } else {
            self.odd.value(u, v, p)
        }
    }

    fn to_spec(&self, registry: &mut ResourceRegistry) -> TextureSpec {
        let even = self.even.to_spec(registry);
        registry.register_texture(self.even.name().to_owned(), even);

        let odd = self.odd.to_spec(registry);
        registry.register_texture(self.odd.name().to_owned(), odd);

        TextureSpec::Checker {
            scale: 1.0 / self.inv_scale,
            even: self.even.name().to_owned(),
            odd: self.odd.name().to_owned(),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}
