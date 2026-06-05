//! Exporter Scene IR -> Chosen representation (SVG, etc)

use crate::ir::scene::Scene;

mod svg_export;

pub struct Export<T> {
    pub name: String,
    pub data: T,
}

pub fn export_scene_svg(scene: Scene) -> Vec<Export<String>> {
    let Scene { exports, settings } = scene;
    exports
        .into_iter()
        .enumerate()
        .map(|(i, graphic)| Export {
            name: format!("export_{}", i),
            data: svg_export::export_to_svg(graphic, settings),
        })
        .collect()
}

/// Formats a float for string output: rounds to `precision` decimal places, then
/// strips any trailing zeros and a trailing decimal point (`100.000` -> `100`,
/// `0.35355` -> `0.354`). Normalizes a bare `-0` to `0`.
pub fn fmt_num(v: f64, precision: usize) -> String {
    let mut s = format!("{:.*}", precision, v);
    if s.contains('.') {
        s.truncate(s.trim_end_matches('0').trim_end_matches('.').len());
    }
    if s == "-0" {
        s = "0".into();
    }
    s
}
