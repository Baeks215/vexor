// Translator for svg export

use crate::ir::scene::{Color, Graphic, GraphicType, Stroke, Style};
use kurbo::Affine;
use svg::node::element as svg_el;

type Attribute = (&'static str, String);

pub fn export_to_svg(graphic: Graphic) -> String {
    translate_graphic(svg::Document::new(), graphic).to_string()
}

/// Applies vector of attributes to an SVG node
macro_rules! apply_attributes {
    ($node:expr, $attributes:expr) => {{
        let mut node = $node;
        for (key, value) in $attributes {
            node = node.set(key, value);
        }
        node
    }};
}

trait Appendable {
    fn add<T>(self, node: T) -> Self
    where
        T: Into<Box<dyn svg::Node>>;
}
impl Appendable for svg::Document {
    fn add<T>(self, node: T) -> Self
    where
        T: Into<Box<dyn svg::Node>>,
    {
        self.add(node)
    }
}
impl Appendable for svg_el::Group {
    fn add<T>(self, node: T) -> Self
    where
        T: Into<Box<dyn svg::Node>>,
    {
        self.add(node)
    }
}

/// Translates a graphic to an SVG document node
fn translate_graphic<T: Appendable>(current: T, graphic: Graphic) -> T {
    let Graphic {
        ty,
        style,
        transform,
    } = graphic;

    let mut extra = vec![];
    transform.add_as_attr(&mut extra);
    style.add_as_attr(&mut extra);

    match ty {
        GraphicType::Circle { radius } => current.add(apply_attributes!(
            svg_el::Circle::new()
                .set("r", radius)
                .set("cx", 0.0)
                .set("cy", 0.0),
            extra
        )),
        GraphicType::Rect { width, height } => current.add(apply_attributes!(
            svg_el::Rectangle::new()
                .set("width", width)
                .set("height", height)
                .set("x", 0.0)
                .set("y", 0.0),
            extra
        )),
        GraphicType::Text { content } => current.add(apply_attributes!(
            svg_el::Text::new(content).set("x", 0.0).set("y", 0.0),
            extra
        )),
        GraphicType::Group { children } => {
            let mut group_node = svg_el::Group::new();
            for child in children {
                group_node = translate_graphic(group_node, child);
            }
            current.add(apply_attributes!(group_node, extra))
        }
    }
}

trait ToAttributes {
    /// Converts to svg attributes and adds to the vector
    fn add_as_attr(self, current: &mut Vec<Attribute>);
}
impl ToAttributes for Affine {
    fn add_as_attr(self, current: &mut Vec<Attribute>) {
        if self == Affine::IDENTITY {
            return;
        }
        let [a, b, c, d, e, f] = self.as_coeffs();
        current.push((
            "transform",
            format!("matrix({} {} {} {} {} {})", a, b, c, d, e, f),
        ));
        current.push(("vector-effect", "non-scaling-stroke".to_string()))
    }
}
impl ToAttributes for Style {
    fn add_as_attr(self, current: &mut Vec<Attribute>) {
        let Style { fill, stroke } = self;
        current.push(("fill", color_to_svg(fill)));

        if let Some(Stroke { color, width }) = stroke {
            current.push(("stroke", color_to_svg(color)));
            current.push(("stroke-width", width.to_string()));
        }
    }
}

/// Converts a color to an SVG color string
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
