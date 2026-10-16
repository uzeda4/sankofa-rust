//! Milestone 0 spike: can an egui/wgpu app on Android show a grid of real covers, tilt them in 3D
//! and take text input, while scrolling at 60 fps?
mod covers;
mod tilt;

#[cfg(target_os = "android")]
mod app {
    use crate::{covers, tilt};
    use eframe::egui::{self, Color32, ColorImage, Mesh, Pos2, Rect, Sense, Shape, Stroke, TextureHandle, vec2};
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::time::Instant;

    const SOURCE: &str = "https://www.mangakakalot.gg/";
    const COVERS: usize = 60;
    const WORKERS: usize = 4;
    /// Covers are decoded and shrunk to this width, like a grid thumbnail cache would.
    const THUMB_W: u32 = 360;

    enum Msg {
        Count(usize),
        Image { index: usize, image: ColorImage, decode_ms: f32 },
        Failed(String),
    }

    pub struct Spike {
        query: String,
        frame_ms: f32,
        worst_ms: f32,
        covers: Vec<Option<TextureHandle>>,
        rx: Receiver<Msg>,
        started: Instant,
        first_cover_s: Option<f32>,
        all_covers_s: Option<f32>,
        loaded: usize,
        decode_ms_sum: f32,
        failures: Vec<String>,
        window: Vec<f32>,
    }

    impl Spike {
        pub fn new(ctx: egui::Context) -> Self {
            let (tx, rx) = channel();
            std::thread::spawn(move || fetch_all(ctx, tx));
            Self {
                query: String::new(),
                frame_ms: 16.0,
                worst_ms: 0.0,
                covers: Vec::new(),
                rx,
                started: Instant::now(),
                first_cover_s: None,
                all_covers_s: None,
                loaded: 0,
                decode_ms_sum: 0.0,
                failures: Vec::new(),
                window: Vec::new(),
            }
        }

        fn drain(&mut self, ctx: &egui::Context) {
            while let Ok(msg) = self.rx.try_recv() {
                match msg {
                    Msg::Count(n) => self.covers = (0..n).map(|_| None).collect(),
                    Msg::Image { index, image, decode_ms } => {
                        let tex = ctx.load_texture(format!("cover{index}"), image, egui::TextureOptions::LINEAR);
                        if let Some(slot) = self.covers.get_mut(index) {
                            *slot = Some(tex);
                        }
                        self.loaded += 1;
                        self.decode_ms_sum += decode_ms;
                        let t = self.started.elapsed().as_secs_f32();
                        self.first_cover_s.get_or_insert(t);
                        if self.loaded == self.covers.len() {
                            self.all_covers_s = Some(t);
                        }
                    }
                    Msg::Failed(e) => {
                        log::warn!("cover failed: {e}");
                        self.failures.push(e);
                    }
                }
            }
        }
    }

    /// Where covers pushed by `adb` live; used when the network is not available to the app.
    const LOCAL_COVERS: &str = "/sdcard/Android/data/app.sankofa.rust.spike/files/covers";

    enum Source {
        Url(String),
        File(std::path::PathBuf),
    }

    fn local_covers() -> Vec<Source> {
        let mut files: Vec<_> = std::fs::read_dir(LOCAL_COVERS)
            .map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).collect())
            .unwrap_or_default();
        files.sort();
        files.truncate(COVERS);
        files.into_iter().map(Source::File).collect()
    }

    fn fetch_all(ctx: egui::Context, tx: Sender<Msg>) {
        let client = sk_net::Client::new();
        let sources: Vec<Source> = match client.get_text(SOURCE) {
            Ok(html) => covers::extract_image_urls(&html, COVERS).into_iter().map(Source::Url).collect(),
            Err(e) => {
                let _ = tx.send(Msg::Failed(format!("listing: {e}")));
                local_covers()
            }
        };
        let _ = tx.send(Msg::Count(sources.len()));
        ctx.request_repaint();
        std::thread::scope(|s| {
            for w in 0..WORKERS {
                let (sources, tx, ctx, client) = (&sources, tx.clone(), ctx.clone(), client.clone());
                s.spawn(move || {
                    for index in (w..sources.len()).step_by(WORKERS) {
                        match load_cover(&client, &sources[index]) {
                            Ok((image, decode_ms)) => {
                                let _ = tx.send(Msg::Image { index, image, decode_ms });
                            }
                            Err(e) => {
                                let _ = tx.send(Msg::Failed(e));
                            }
                        }
                        ctx.request_repaint();
                    }
                });
            }
        });
    }

    fn load_cover(client: &sk_net::Client, source: &Source) -> Result<(ColorImage, f32), String> {
        let bytes = match source {
            Source::Url(url) => client
                .get_bytes_with(url, &[("Referer", SOURCE)])
                .map_err(|e| format!("{url}: {e}"))?,
            Source::File(path) => std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?,
        };
        let t = Instant::now();
        let img = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
        let h = (img.height() as f32 * THUMB_W as f32 / img.width() as f32).round().max(1.0) as u32;
        let rgba = image::imageops::thumbnail(&img.to_rgba8(), THUMB_W, h);
        let size = [rgba.width() as usize, rgba.height() as usize];
        let image = ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
        Ok((image, t.elapsed().as_secs_f32() * 1000.0))
    }

    impl eframe::App for Spike {
        fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
            let ctx = ui.ctx().clone();
            self.drain(&ctx);
            let dt = ctx.input(|i| i.unstable_dt) * 1000.0;
            self.frame_ms = self.frame_ms * 0.95 + dt * 0.05;
            self.worst_ms = (self.worst_ms * 0.995).max(dt);
            // Every 120 frames, log the frame-time spread so scrolling can be judged without screenshots
            // (a screenshot itself causes a slow frame).
            self.window.push(dt);
            if self.window.len() >= 120 {
                self.window.sort_by(|a, b| a.total_cmp(b));
                let w = &self.window;
                log::info!(
                    "frames p50 {:.1} p95 {:.1} p99 {:.1} max {:.1} ms, >20ms: {}",
                    w[w.len() / 2],
                    w[w.len() * 95 / 100],
                    w[w.len() * 99 / 100],
                    w[w.len() - 1],
                    w.iter().filter(|&&x| x > 20.0).count()
                );
                self.window.clear();
            }

            egui::Panel::top("bar").show_inside(ui, |ui| {
                ui.add_space(24.0);
                ui.label(format!("{:.1} ms avg, {:.1} ms worst", self.frame_ms, self.worst_ms));
                ui.label(format!(
                    "{}/{} covers · first {} · all {} · decode {:.0} ms/img · {} failed",
                    self.loaded,
                    self.covers.len(),
                    self.first_cover_s.map_or("-".into(), |t| format!("{t:.1}s")),
                    self.all_covers_s.map_or("-".into(), |t| format!("{t:.1}s")),
                    self.decode_ms_sum / self.loaded.max(1) as f32,
                    self.failures.len(),
                ));
                ui.text_edit_singleline(&mut self.query);
            });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let columns = 3;
                    let w = (ui.available_width() - 8.0 * (columns as f32 + 1.0)) / columns as f32;
                    let h = w * 1.5;
                    let rows = self.covers.len().div_ceil(columns);
                    for row in 0..rows {
                        ui.horizontal(|ui| {
                            for col in 0..columns {
                                let n = row * columns + col;
                                if n >= self.covers.len() {
                                    break;
                                }
                                let (rect, resp) = ui.allocate_exact_size(vec2(w, h), Sense::click_and_drag());
                                if !ui.is_rect_visible(rect) {
                                    continue;
                                }
                                let (ax, ay) = match resp.interact_pointer_pos() {
                                    Some(p) if resp.is_pointer_button_down_on() => {
                                        let f = (p - rect.center()) / vec2(w, h) * 2.0;
                                        (-f.y * 0.25, f.x * 0.25)
                                    }
                                    _ => (0.0, 0.0),
                                };
                                draw_cover(ui, rect, self.covers[n].as_ref(), ax, ay);
                            }
                        });
                    }
                });
            });
            // Keep the frame loop running so the frame-time readout is meaningful.
            ctx.request_repaint();
        }
    }

    /// Draws the tilted cover; a flat placeholder while it loads.
    fn draw_cover(ui: &egui::Ui, rect: Rect, tex: Option<&TextureHandle>, ax: f32, ay: f32) {
        let c = tilt::project([rect.left(), rect.top()], [rect.right(), rect.bottom()], ax, ay, 900.0)
            .map(|p| Pos2::new(p[0], p[1]));
        match tex {
            Some(tex) => {
                // ponytail: two triangles, so the texture is affine-mapped; a finer mesh if the tilt
                // gets stronger and the distortion shows.
                let mut mesh = Mesh::with_texture(tex.id());
                let uv = [Pos2::new(0.0, 0.0), Pos2::new(1.0, 0.0), Pos2::new(1.0, 1.0), Pos2::new(0.0, 1.0)];
                for i in 0..4 {
                    mesh.vertices.push(egui::epaint::Vertex { pos: c[i], uv: uv[i], color: Color32::WHITE });
                }
                mesh.add_triangle(0, 1, 2);
                mesh.add_triangle(0, 2, 3);
                ui.painter().add(Shape::mesh(mesh));
            }
            None => {
                ui.painter().add(Shape::convex_polygon(c.to_vec(), Color32::from_gray(48), Stroke::NONE));
            }
        }
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));
    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native(
        "sk-ui-spike",
        options,
        Box::new(|cc| Ok(Box::new(app::Spike::new(cc.egui_ctx.clone())))),
    ) {
        log::error!("run_native failed: {e}");
    }
}
