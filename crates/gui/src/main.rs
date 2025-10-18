use std::{
    fs::File,
    io::BufReader,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc::{Receiver, Sender, channel},
    },
};

use eframe::egui::{self, ImageSource};
use log::error;
use ray_tracer::{
    camera::{Camera, PPMRenderWriter, RenderProgressTracker},
    hittable::HittableList,
    scene_loader::SceneFile,
    vec::{Point3, Vec3},
};

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
            Ok(Box::new(RtiowApp::new()))
        }),
    )
    .map_err(|e| anyhow::format_err!("{e}"))?;

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct JobId(u64);

impl JobId {
    fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

struct JobRequest {
    id: JobId,
    params: RenderJob,
}

struct JobResult {
    id: JobId,
    image: Arc<[u8]>,
}

#[derive(Debug, Clone, PartialEq)]
struct RenderJob {
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
}

impl Default for RenderJob {
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
        }
    }
}

struct RenderProgressState {
    total: AtomicUsize,
    current: AtomicUsize,
}

impl RenderProgressState {
    pub fn new() -> Self {
        Self {
            total: AtomicUsize::new(0),
            current: AtomicUsize::new(0),
        }
    }

    pub fn progress(&self) -> f32 {
        let total = self.total.load(Ordering::Relaxed);
        let current = self.current.load(Ordering::Relaxed);

        current as f32 / total as f32
    }
}

impl RenderProgressTracker for RenderProgressState {
    fn init(&self, total: usize) {
        self.total.swap(total, Ordering::Relaxed);
    }

    fn tick(&self, _current: usize) {
        self.current.fetch_add(1, Ordering::SeqCst);
    }
}

struct RtiowApp {
    job_params: RenderJob,
    last_sent_params: RenderJob,
    render_progress: Option<Arc<RenderProgressState>>,
    next_job_id: JobId,
    newest_requested_job: Option<JobId>,
    newest_finished_job: Option<JobId>,
    job_tx: Sender<(JobRequest, Arc<RenderProgressState>)>,
    result_rx: Receiver<JobResult>,
    image_bytes: Option<Arc<[u8]>>,
}

const IMAGE_URI: &str = "bytes://rendered.ppm";

impl RtiowApp {
    pub fn new() -> Self {
        let (job_tx, job_rx) = channel::<(JobRequest, Arc<RenderProgressState>)>();
        let (result_tx, result_rx) = channel::<JobResult>();

        let file = File::open("scenes/cover.json").unwrap();
        let reader = BufReader::new(file);
        let scene: SceneFile = serde_json::from_reader(reader).unwrap();
        let world = scene.into_list().unwrap();

        std::thread::spawn(move || {
            while let Ok(mut job) = job_rx.recv() {
                // Drain any queued jobs so we only render latest
                while let Ok(next) = job_rx.try_recv() {
                    job = next;
                }

                let (request, progress) = job;
                let image = render_scene(&request.params, &world, progress);

                if let Err(e) = result_tx.send(JobResult {
                    id: request.id,
                    image,
                }) {
                    error!("render thread closed: {e}")
                }
            }
        });

        Self {
            job_params: RenderJob::default(),
            last_sent_params: RenderJob::default(),
            render_progress: None,
            next_job_id: JobId(0),
            newest_requested_job: None,
            newest_finished_job: None,
            job_tx,
            result_rx,
            image_bytes: None,
        }
    }

    fn control_panel(&mut self, ui: &mut eframe::egui::Ui) {
        ui.heading("Ray tracing in one weekend");
        ui.add_space(20.0);

        ui.spacing_mut().item_spacing = egui::vec2(10.0, 15.0);

        ui.horizontal(|ui| {
            let label = ui.label("aspect ratio");
            ui.add(
                egui::DragValue::new(&mut self.job_params.aspect_ratio)
                    .speed(0.1)
                    .range(0.1..=f64::INFINITY),
            )
            .labelled_by(label.id);
        });

        ui.horizontal(|ui| {
            let label = ui.label("image width");
            ui.add(
                egui::DragValue::new(&mut self.job_params.image_width)
                    .speed(1)
                    .range(1..=i32::MAX),
            )
            .labelled_by(label.id);
        });

        ui.horizontal(|ui| {
            let label = ui.label("samples");
            ui.add(
                egui::DragValue::new(&mut self.job_params.samples_per_pixel)
                    .speed(1)
                    .range(1..=i32::MAX),
            )
            .labelled_by(label.id);
        });

        ui.horizontal(|ui| {
            let label = ui.label("max depth");
            ui.add(
                egui::DragValue::new(&mut self.job_params.max_depth)
                    .speed(1)
                    .range(1..=i32::MAX),
            )
            .labelled_by(label.id);
        });

        ui.horizontal(|ui| {
            let label = ui.label("vfov");
            ui.add(egui::Slider::new(&mut self.job_params.vfov, 0.0..=360.0))
                .labelled_by(label.id);
        });

        vector_input(ui, "lookfrom", &mut self.job_params.lookfrom);
        vector_input(ui, "lookat", &mut self.job_params.lookat);
        vector_input(ui, "vup", &mut self.job_params.vup);

        ui.horizontal(|ui| {
            let label = ui.label("defocus angle");
            ui.add(egui::Slider::new(
                &mut self.job_params.defocus_angle,
                0.0..=360.0,
            ))
            .labelled_by(label.id);
        });

        ui.horizontal(|ui| {
            let label = ui.label("focus dist");
            ui.add(
                egui::DragValue::new(&mut self.job_params.focus_dist)
                    .speed(1.0)
                    .range(1.0..=f64::INFINITY),
            )
            .labelled_by(label.id);
        });

        ui.separator();

        if let Some(progress) = &self.render_progress {
            ui.add(egui::ProgressBar::new(progress.progress()));
        }
    }

    fn render_panel(&mut self, ui: &mut eframe::egui::Ui) {
        if let Some(image) = &self.image_bytes {
            ui.image(ImageSource::Bytes {
                uri: IMAGE_URI.into(),
                bytes: image.clone().into(),
            });
        }
    }
}

impl eframe::App for RtiowApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(result) = self.result_rx.try_recv() {
            if Some(result.id) >= self.newest_requested_job {
                self.render_progress = None;

                ctx.forget_image(IMAGE_URI);
                self.image_bytes = Some(result.image);
                self.newest_finished_job = Some(result.id);
                ctx.request_repaint();
            }
        }

        egui::SidePanel::left("camera").show(ctx, |ui| {
            self.control_panel(ui);
        });

        // push new render job if params changed
        if self.job_params != self.last_sent_params {
            self.next_job_id = self.next_job_id.next();
            let job_id = self.next_job_id;
            self.last_sent_params = self.job_params.clone();
            self.newest_requested_job = Some(job_id);

            let progress = Arc::new(RenderProgressState::new());
            self.render_progress = Some(progress.clone());

            if self
                .job_tx
                .send((
                    JobRequest {
                        id: job_id,
                        params: self.job_params.clone(),
                    },
                    progress,
                ))
                .is_ok()
            {
                ctx.request_repaint();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_panel(ui);
        });

        // poll until latest job finishes
        if self.newest_finished_job != self.newest_requested_job {
            ctx.request_repaint();
        }
    }
}

fn render_scene(
    params: &RenderJob,
    world: &HittableList,
    progress_tracker: Arc<RenderProgressState>,
) -> Arc<[u8]> {
    let camera = Camera::builder()
        .image_width(params.image_width)
        .aspect_ratio(params.aspect_ratio)
        .samples_per_pixel(params.samples_per_pixel)
        .max_depth(params.max_depth)
        .vfov(params.vfov)
        .lookfrom(params.lookfrom.clone())
        .lookat(params.lookat.clone())
        .vup(params.vup.clone())
        .defocus_angle(params.defocus_angle)
        .focus_dist(params.focus_dist)
        .build();

    let out: Vec<u8> = Vec::new();
    let mut out = PPMRenderWriter::new(out);
    if let Err(e) = camera.render(world, &mut out, progress_tracker.as_ref()) {
        error!("render error: {e}")
    };

    out.take().into_boxed_slice().into()
}

fn vector_input(ui: &mut eframe::egui::Ui, label: &str, vec: &mut Vec3) {
    ui.horizontal(|ui| {
        let label = ui.label(label);
        ui.add(egui::DragValue::new(vec.x_mut()).speed(0.1))
            .labelled_by(label.id);
        ui.add(egui::DragValue::new(vec.y_mut()).speed(0.1))
            .labelled_by(label.id);
        ui.add(egui::DragValue::new(vec.z_mut()).speed(0.1))
            .labelled_by(label.id);
    });
}
