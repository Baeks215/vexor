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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::scene::Graphic;

    #[test]
    fn test_empty_scene() {
        let scene = Scene { exports: vec![] };
        let result = export_scene_svg(scene);
        assert!(result.is_empty());
    }

    fn red() -> crate::ir::scene::Color {
        crate::ir::scene::Color::Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    #[test]
    fn test_single_export() {
        let scene = Scene {
            exports: vec![Graphic::Circle {
                x: 0.0,
                y: 0.0,
                radius: 10.0,
                color: red(),
            }],
        };
        let result = export_scene_svg(scene);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "export_0");
        assert!(result[0].data.contains("<circle"));
    }

    #[test]
    fn test_multiple_exports_naming() {
        let scene = Scene {
            exports: vec![
                Graphic::Circle {
                    x: 0.0,
                    y: 0.0,
                    radius: 1.0,
                    color: red(),
                },
                Graphic::Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 2.0,
                    height: 3.0,
                    color: red(),
                },
                Graphic::Text {
                    x: 0.0,
                    y: 0.0,
                    content: "hi".to_string(),
                    color: red(),
                },
            ],
        };
        let result = export_scene_svg(scene);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].name, "export_0");
        assert_eq!(result[1].name, "export_1");
        assert_eq!(result[2].name, "export_2");
        assert!(result[0].data.contains("<circle"));
        assert!(result[1].data.contains("<rect"));
        assert!(result[2].data.contains("<text"));
        assert!(result[2].data.contains("hi"));
    }
}
