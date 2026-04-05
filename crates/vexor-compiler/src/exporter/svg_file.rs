use std::path::Path;

use crate::ir::scene::Graphic;
use svg::node::element as svg_el;

/// Exports a scene to a SVG strings.
pub fn export_to_svg(
    graphic: Graphic,
    target_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let doc = translate_graphic(svg::Document::new(), graphic);
    svg::save(target_path, &doc)?;
    Ok(())
}

/// Translates a graphic to an SVG node.
fn translate_graphic(current: svg::Document, graphic: Graphic) -> svg::Document {
    match graphic {
        Graphic::Circle { radius } => {
            let node = svg_el::Circle::new().set("radius", radius);
            current.add(node)
        }
        Graphic::Rect { width, height } => {
            let node = svg_el::Rectangle::new()
                .set("width", width)
                .set("height", height);
            current.add(node)
        }
        Graphic::Text(content) => {
            let node = svg_el::Text::new(content);
            current.add(node)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle() {
        let doc = translate_graphic(svg::Document::new(), Graphic::Circle { radius: 42.0 });
        let svg = doc.to_string();
        assert!(svg.contains("<circle"));
        assert!(svg.contains("radius=\"42\""));
    }

    #[test]
    fn test_rect() {
        let doc = translate_graphic(
            svg::Document::new(),
            Graphic::Rect {
                width: 100.0,
                height: 50.0,
            },
        );
        let svg = doc.to_string();
        assert!(svg.contains("<rect"));
        assert!(svg.contains("width=\"100\""));
        assert!(svg.contains("height=\"50\""));
    }

    #[test]
    fn test_text() {
        let doc = translate_graphic(
            svg::Document::new(),
            Graphic::Text("hello".to_string()),
        );
        let svg = doc.to_string();
        assert!(svg.contains("<text"));
        assert!(svg.contains("hello"));
    }
}
