use std::path::Path;

use crate::ir::scene::Scene;

mod svg_file;

pub enum ExportType {
    Svg,
}

pub fn export_scene(scene: Scene) -> Result<(), Box<dyn std::error::Error>> {
    let Scene { exports } = scene;
    let mut i = 0;
    for graphic in exports {
        i += 1;
        let path_string = format!("export_{}.svg", i);
        let path = Path::new(&path_string);
        svg_file::export_to_svg(graphic, &path)?
    }
    Ok(())
}
