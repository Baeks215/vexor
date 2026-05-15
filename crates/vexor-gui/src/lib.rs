use eframe::egui;
use std::sync::mpsc;

const IMAGE_WIDTH_FRACTION: f32 = 2.0 / 3.0;

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
            }))
        }),
    )
}

struct GuiApp {
    rx: mpsc::Receiver<Vec<NamedSvg>>,
    exports: Vec<NamedSvg>,
    version: u64,
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
        let image_width = ui.available_width() * IMAGE_WIDTH_FRACTION;
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                for ex in &self.exports {
                    ui.group(|ui| {
                        ui.set_width(image_width);
                        ui.vertical_centered(|ui| {
                            ui.heading(&ex.name);
                            let uri = format!("bytes://v{}_{}.svg", self.version, ex.name);
                            ui.add(
                                egui::Image::from_bytes(uri, ex.svg.as_bytes().to_vec())
                                    .fit_to_fraction(egui::Vec2::new(1.0, f32::INFINITY)),
                            );
                        });
                    });
                    ui.add_space(8.0);
                }
            });
        });
    }
}
