//! Milestone 0 spike: can an egui/wgpu app on Android do a cover grid, a 3D tilt and text input?
mod tilt;

#[cfg(target_os = "android")]
use eframe::egui::{self, Color32, Pos2, Sense, Shape, Stroke, vec2};

#[cfg(target_os = "android")]
struct Spike {
    query: String,
    frame_ms: f32,
    worst_ms: f32,
}

#[cfg(target_os = "android")]
impl eframe::App for Spike {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let dt = ctx.input(|i| i.unstable_dt) * 1000.0;
        self.frame_ms = self.frame_ms * 0.95 + dt * 0.05;
        self.worst_ms = (self.worst_ms * 0.995).max(dt);

        egui::Panel::top("bar").show_inside(ui, |ui| {
            ui.add_space(24.0);
            ui.horizontal(|ui| {
                ui.label("Sankofa Rust spike");
                ui.label(format!("{:.1} ms avg, {:.1} ms worst", self.frame_ms, self.worst_ms));
            });
            ui.text_edit_singleline(&mut self.query);
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let columns = 3;
                let w = (ui.available_width() - 8.0 * (columns as f32 + 1.0)) / columns as f32;
                let h = w * 1.5;
                for row in 0..40 {
                    ui.horizontal(|ui| {
                        for col in 0..columns {
                            let n = row * columns + col;
                            let (rect, resp) = ui.allocate_exact_size(vec2(w, h), Sense::click_and_drag());
                            let (ax, ay) = match resp.interact_pointer_pos() {
                                Some(p) if resp.is_pointer_button_down_on() => {
                                    let f = (p - rect.center()) / vec2(w, h) * 2.0;
                                    (-f.y * 0.25, f.x * 0.25)
                                }
                                _ => (0.0, 0.0),
                            };
                            let pts: Vec<Pos2> = tilt::project(
                                [rect.left(), rect.top()],
                                [rect.right(), rect.bottom()],
                                ax,
                                ay,
                                900.0,
                            )
                            .iter()
                            .map(|p| Pos2::new(p[0], p[1]))
                            .collect();
                            let hue = (n as f32 * 0.137).fract();
                            let fill: Color32 = egui::ecolor::Hsva::new(hue, 0.55, 0.45, 1.0).into();
                            ui.painter().add(Shape::convex_polygon(pts, fill, Stroke::NONE));
                            ui.painter().text(
                                Pos2::new(rect.left() + 6.0, rect.bottom() - 6.0),
                                egui::Align2::LEFT_BOTTOM,
                                format!("Title {n}"),
                                egui::FontId::proportional(14.0),
                                Color32::WHITE,
                            );
                        }
                    });
                }
            });
        });
        // Keep the frame loop running so the frame-time readout is meaningful.
        ctx.request_repaint();
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
    let spike = Spike { query: String::new(), frame_ms: 16.0, worst_ms: 0.0 };
    if let Err(e) = eframe::run_native("sk-ui-spike", options, Box::new(|_| Ok(Box::new(spike)))) {
        log::error!("run_native failed: {e}");
    }
}
