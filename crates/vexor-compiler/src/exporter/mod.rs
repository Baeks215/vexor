//! Exporter Scene IR -> Chosen representation (SVG, etc)

use kurbo::Point;

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
    let mut s = String::new();
    write_num(&mut s, v, precision);
    s
}

/// Appends a formatted float to `out`.
pub fn write_num(out: &mut String, v: f64, precision: usize) {
    use std::fmt::Write;
    let start = out.len();
    let _ = write!(out, "{:.*}", precision, v);
    // Strip trailing zeros and a trailing decimal point, in place.
    if out[start..].contains('.') {
        let trimmed = out[start..]
            .trim_end_matches('0')
            .trim_end_matches('.')
            .len();
        out.truncate(start + trimmed);
    }
    // Normalize a bare `-0` to `0`.
    if &out[start..] == "-0" {
        out.truncate(start);
        out.push('0');
    }
}

/// Appends a point as `x,y` to `out`, formatting each coordinate with [`write_num`].
pub fn write_point(out: &mut String, p: Point, precision: usize) {
    write_num(out, p.x, precision);
    out.push(',');
    write_num(out, p.y, precision);
}
