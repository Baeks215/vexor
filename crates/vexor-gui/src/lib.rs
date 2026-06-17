mod svg_loader;

use eframe::egui;
use std::sync::mpsc;

const IMAGE_WIDTH_FRACTION: f32 = 2.0 / 3.0;
/// Raster width is snapped to multiples of this so resizing reuses cached textures.
const RASTER_STEP: f32 = 256.0;

pub struct NamedSvg {
    pub name: String,
    pub svg: String,
}

pub fn run(
    title: String,
    rx: mpsc::Receiver<Vec<NamedSvg>>,
    setup: impl FnOnce(egui::Context) + Send + 'static,
) -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    let mut setup = Some(setup);
    eframe::run_native(
        &title,
        options,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            cc.egui_ctx
                .add_image_loader(std::sync::Arc::new(svg_loader::VexorSvgLoader::new()));
            let mut visuals = egui::Visuals::light();
            visuals.panel_fill = egui::Color32::WHITE;
            visuals.window_fill = egui::Color32::WHITE;
            cc.egui_ctx.set_visuals(visuals);
            if let Some(setup) = setup.take() {
                setup(cc.egui_ctx.clone());
            }
            Ok(Box::new(GuiApp {
                rx,
                exports: Vec::new(),
                version: 0,
                focus: None,
            }))
        }),
    )
}

struct GuiApp {
    rx: mpsc::Receiver<Vec<NamedSvg>>,
    exports: Vec<NamedSvg>,
    version: u64,
    focus: Option<usize>,
}

impl eframe::App for GuiApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [1.0, 1.0, 1.0, 1.0]
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        while let Ok(exports) = self.rx.try_recv() {
            self.exports = exports;
            self.version = self.version.wrapping_add(1);
        }
        if let Some(idx) = self.focus {
            if idx < self.exports.len() {
                self.focus_view(ui, idx);
                return;
            }
            self.set_focus(ui, None);
        }
        let image_width = ui.available_width() * IMAGE_WIDTH_FRACTION;
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                for idx in 0..self.exports.len() {
                    let (name, svg_bytes) = {
                        let ex = &self.exports[idx];
                        (ex.name.clone(), ex.svg.as_bytes().to_vec())
                    };
                    ui.group(|ui| {
                        ui.set_width(image_width);
                        ui.vertical_centered(|ui| {
                            let mut focus = false;
                            ui.horizontal(|ui| {
                                ui.heading(&name);
                                focus = ui.button("Focus").clicked();
                            });
                            let uri = format!("bytes://v{}_{}.svg", self.version, name);
                            let image = egui::Image::from_bytes(uri, svg_bytes);
                            // Snap the raster width to coarse steps so most resize frames reuse
                            // a cached texture instead of re-rasterizing the SVG every pixel.
                            let snapped = (image_width / RASTER_STEP).ceil() * RASTER_STEP;
                            if let Ok(egui::load::TexturePoll::Ready { texture }) =
                                image.load_for_size(ui.ctx(), egui::vec2(snapped, f32::INFINITY))
                            {
                                let h = image_width * texture.size.y / texture.size.x;
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(image_width, h),
                                    egui::Sense::hover(),
                                );
                                let uv = egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                );
                                ui.painter()
                                    .image(texture.id, rect, uv, egui::Color32::WHITE);
                            }
                            if focus {
                                self.set_focus(ui, Some(idx));
                            }
                        });
                    });
                    ui.add_space(8.0);
                }
            });
        });
    }
}

impl GuiApp {
    /// Enter/exit focus mode, taking the window in/out of real monitor fullscreen.
    fn set_focus(&mut self, ui: &egui::Ui, target: Option<usize>) {
        self.focus = target;
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::Fullscreen(target.is_some()));
    }

    fn focus_view(&mut self, ui: &mut egui::Ui, idx: usize) {
        let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
        let mut exit = escape;
        ui.horizontal(|ui| {
            exit |= ui.button("Exit focus").clicked();
        });
        if exit {
            self.set_focus(ui, None);
            return;
        }

        let (name, svg_bytes) = {
            let ex = &self.exports[idx];
            (ex.name.clone(), ex.svg.as_bytes().to_vec())
        };
        let uri = format!("bytes://focus_v{}_{}.svg", self.version, name);
        ui.centered_and_justified(|ui| {
            ui.add(
                egui::Image::from_bytes(uri, svg_bytes).fit_to_fraction(egui::Vec2::new(1.0, 1.0)),
            );
        });
    }
}
