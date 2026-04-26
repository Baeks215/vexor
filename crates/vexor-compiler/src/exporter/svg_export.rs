// Translator for svg export

use crate::ir::scene::{Color, Graphic};
use svg::node::element as svg_el;

fn color_to_svg(color: Color) -> String {
    match color {
        Color::Rgba { r, g, b, a } => format!(
            "rgba({},{},{},{})",
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8,
            a
        ),
    }
}

/// Translates a graphic into an SVG document string.
pub fn export_to_svg(graphic: Graphic) -> String {
    translate_graphic(svg::Document::new(), graphic).to_string()
}

/// Translates a graphic to an SVG node.
fn translate_graphic(current: svg::Document, graphic: Graphic) -> svg::Document {
    match graphic {
        Graphic::Circle {
            x,
            y,
            radius,
            color,
        } => {
            let node = svg_el::Circle::new()
                .set("cx", x)
                .set("cy", y)
                .set("r", radius)
                .set("fill", color_to_svg(color));
            current.add(node)
        }
        Graphic::Rect {
            x,
            y,
            width,
            height,
            color,
        } => {
            let node = svg_el::Rectangle::new()
                .set("x", x)
                .set("y", y)
                .set("width", width)
                .set("height", height)
                .set("fill", color_to_svg(color));
            current.add(node)
        }
        Graphic::Text {
            x,
            y,
            content,
            color,
        } => {
            let node = svg_el::Text::new(content)
                .set("x", x)
                .set("y", y)
                .set("fill", color_to_svg(color));
            current.add(node)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn red() -> Color {
        Color::Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    #[test]
    fn test_circle() {
        let doc = translate_graphic(
            svg::Document::new(),
            Graphic::Circle {
                x: 10.0,
                y: 20.0,
                radius: 42.0,
                color: red(),
            },
        );
        let svg = doc.to_string();
        assert!(svg.contains("<circle"));
        assert!(svg.contains("r=\"42\""));
        assert!(svg.contains("cx=\"10\""));
        assert!(svg.contains("cy=\"20\""));
    }

    #[test]
    fn test_rect() {
        let doc = translate_graphic(
            svg::Document::new(),
            Graphic::Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
                color: red(),
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
            Graphic::Text {
                x: 0.0,
                y: 0.0,
                content: "hello".to_string(),
                color: red(),
            },
        );
        let svg = doc.to_string();
        assert!(svg.contains("<text"));
        assert!(svg.contains("hello"));
    }
}
