//! Exporter Scene IR -> Chosen representation (SVG, etc)

use crate::ir::scene::Scene;

mod svg_export;

pub struct Export<T> {
    pub name: String,
    pub data: T,
}

pub fn export_scene_svg(scene: Scene) -> Vec<Export<String>> {
    let Scene { exports } = scene;
    exports
        .into_iter()
        .enumerate()
        .map(|(i, graphic)| Export {
            name: format!("export_{}", i),
            data: svg_export::export_to_svg(graphic),
        })
        .collect()
}
