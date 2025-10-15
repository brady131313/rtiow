use std::sync::Arc;

use eframe::egui::{self, ImageSource};
use ray_tracer::vec::{Point3, Vec3};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        ..Default::default()
    };

    eframe::run_native(
        "rtiow",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<RtiowApp>::default())
        }),
    )
    .map_err(|e| anyhow::format_err!("{e}"))?;

    Ok(())
}

struct RtiowApp {
    aspect_ratio: f64,
    image_width: i32,
    samples_per_pixel: i32,
    max_depth: i32,
    vfov: f64,
    lookfrom: Point3,
    lookat: Point3,
    vup: Vec3,
    defocus_angle: f64,
    focus_dist: f64,
    image_bytes: Option<Arc<[u8]>>,
}

impl Default for RtiowApp {
    fn default() -> Self {
        Self {
            aspect_ratio: 16.0 / 9.0,
            image_width: 400,
            samples_per_pixel: 50,
            max_depth: 10,
            vfov: 90.0,
            lookfrom: Point3::ZERO,
            lookat: Point3::new(0.0, 0.0, -1.0),
            vup: Vec3::new(0.0, 1.0, 0.0),
            defocus_angle: 0.0,
            focus_dist: 10.0,
            image_bytes: None,
        }
    }
}

impl eframe::App for RtiowApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("camera").show(ctx, |ui| {
            ui.heading("Ray tracing in one weekend");
            ui.add_space(20.0);

            ui.spacing_mut().item_spacing = egui::vec2(10.0, 15.0);

            ui.horizontal(|ui| {
                let label = ui.label("aspect ratio");
                ui.add(
                    egui::DragValue::new(&mut self.aspect_ratio)
                        .speed(0.1)
                        .range(0.1..=f64::INFINITY),
                )
                .labelled_by(label.id);
            });

            ui.horizontal(|ui| {
                let label = ui.label("image width");
                ui.add(
                    egui::DragValue::new(&mut self.image_width)
                        .speed(1)
                        .range(1..=i32::MAX),
                )
                .labelled_by(label.id);
            });

            ui.horizontal(|ui| {
                let label = ui.label("samples");
                ui.add(
                    egui::DragValue::new(&mut self.samples_per_pixel)
                        .speed(1)
                        .range(1..=i32::MAX),
                )
                .labelled_by(label.id);
            });

            ui.horizontal(|ui| {
                let label = ui.label("max depth");
                ui.add(
                    egui::DragValue::new(&mut self.max_depth)
                        .speed(1)
                        .range(1..=i32::MAX),
                )
                .labelled_by(label.id);
            });

            ui.horizontal(|ui| {
                let label = ui.label("vfov");
                ui.add(egui::Slider::new(&mut self.vfov, 0.0..=360.0))
                    .labelled_by(label.id);
            });

            ui.horizontal(|ui| {
                let label = ui.label("lookfrom");
                ui.add(egui::DragValue::new(self.lookfrom.x_mut()).speed(0.1))
                    .labelled_by(label.id);
                ui.add(egui::DragValue::new(self.lookfrom.y_mut()).speed(0.1))
                    .labelled_by(label.id);
                ui.add(egui::DragValue::new(self.lookfrom.z_mut()).speed(0.1))
                    .labelled_by(label.id);
            });

            ui.horizontal(|ui| {
                let label = ui.label("lookat");
                ui.add(egui::DragValue::new(self.lookat.x_mut()).speed(0.1))
                    .labelled_by(label.id);
                ui.add(egui::DragValue::new(self.lookat.y_mut()).speed(0.1))
                    .labelled_by(label.id);
                ui.add(egui::DragValue::new(self.lookat.z_mut()).speed(0.1))
                    .labelled_by(label.id);
            });

            ui.horizontal(|ui| {
                let label = ui.label("vup");
                ui.add(egui::DragValue::new(self.vup.x_mut()).speed(0.1))
                    .labelled_by(label.id);
                ui.add(egui::DragValue::new(self.vup.y_mut()).speed(0.1))
                    .labelled_by(label.id);
                ui.add(egui::DragValue::new(self.vup.z_mut()).speed(0.1))
                    .labelled_by(label.id);
            });

            ui.horizontal(|ui| {
                let label = ui.label("defocus angle");
                ui.add(egui::Slider::new(&mut self.defocus_angle, 0.0..=360.0))
                    .labelled_by(label.id);
            });

            ui.horizontal(|ui| {
                let label = ui.label("focus dist");
                ui.add(
                    egui::DragValue::new(&mut self.focus_dist)
                        .speed(1.0)
                        .range(1.0..=f64::INFINITY),
                )
                .labelled_by(label.id);
            })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(image) = &self.image_bytes {
                ui.image(ImageSource::Bytes {
                    uri: "bytes://rendered.ppm".into(),
                    bytes: image.clone().into(),
                });
            }
        });
    }
}
