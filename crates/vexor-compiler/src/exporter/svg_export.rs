// Translator for svg export

use crate::exporter::fmt_num;
use crate::ir::scene::{Attr, Color, Graphic, GraphicType, Settings, StrokeCap, StrokeJoin};
use kurbo::Affine;
use svg::node::element as svg_el;

type Attribute = (&'static str, String);

pub fn export_to_svg(graphic: Graphic, settings: Settings) -> String {
    let Settings {
        canvas: (width, height),
        precision,
    } = settings;
    let min_x = -(width as isize) / 2;
    let min_y = -(height as isize) / 2;
    let doc = svg::Document::new()
        .set("width", width)
        .set("height", height)
        .set(
            "viewBox",
            format!("{} {} {} {}", min_x, min_y, width, height),
        );
    translate_graphic(doc, &graphic, precision).to_string()
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
fn translate_graphic<T: Appendable>(current: T, graphic: &Graphic, precision: usize) -> T {
    let Graphic {
        ty,
        attrs,
        transform,
    } = graphic;

    let mut extra = vec![];
    transform.add_as_attr(&mut extra, precision);
    for attr in attrs {
        attr.add_as_attr(&mut extra, precision);
    }
    add_default_style(ty, &mut extra, precision);

    match ty {
        GraphicType::Circle { radius } => current.add(apply_attributes!(
            svg_el::Circle::new().set("r", fmt_num(*radius, precision)),
            extra
        )),
        GraphicType::Ellipse { rx, ry } => current.add(apply_attributes!(
            svg_el::Ellipse::new()
                .set("rx", fmt_num(*rx, precision))
                .set("ry", fmt_num(*ry, precision)),
            extra
        )),
        GraphicType::Rect { width, height } => current.add(apply_attributes!(
            svg_el::Rectangle::new()
                .set("width", fmt_num(*width, precision))
                .set("height", fmt_num(*height, precision)),
            extra
        )),
        GraphicType::Text { content } => {
            current.add(apply_attributes!(svg_el::Text::new(content.clone()), extra))
        }
        GraphicType::Group { children } => {
            let mut group_node = svg_el::Group::new();
            for child in children.iter() {
                group_node = translate_graphic(group_node, child, precision);
            }
            current.add(apply_attributes!(group_node, extra))
        }
        GraphicType::Path { path } => {
            let path_node = svg_el::Path::new().set("d", path.to_svg(precision));
            current.add(apply_attributes!(path_node, extra))
        }
    }
}

/// Applies default styles to shapes:
/// transparent white fill and a 1-unit black stroke.
fn add_default_style(ty: &GraphicType, current: &mut Vec<Attribute>, precision: usize) {
    if !matches!(
        ty,
        GraphicType::Circle { .. }
            | GraphicType::Ellipse { .. }
            | GraphicType::Rect { .. }
            | GraphicType::Path { .. }
    ) {
        return;
    }
    let (mut has_fill, mut has_stroke, mut has_width) = (false, false, false);
    for (key, _) in current.iter() {
        match *key {
            "fill" => has_fill = true,
            "stroke" => has_stroke = true,
            "stroke-width" => has_width = true,
            _ => {}
        }
    }
    if !has_fill {
        Attr::Fill(Color::Rgba {
            r: 255.0,
            g: 255.0,
            b: 255.0,
            a: 0.0,
        })
        .add_as_attr(current, precision);
    }
    if !has_stroke {
        Attr::StrokeColor(Color::Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        })
        .add_as_attr(current, precision);
    }
    if !has_width {
        Attr::StrokeWidth(1.0).add_as_attr(current, precision);
    }
}

trait ToAttributes {
    /// Converts to svg attributes and adds to the vector
    fn add_as_attr(&self, current: &mut Vec<Attribute>, precision: usize);
}
impl ToAttributes for Affine {
    fn add_as_attr(&self, current: &mut Vec<Attribute>, precision: usize) {
        if *self == Affine::IDENTITY {
            return;
        }
        let [a, b, c, d, e, f] = self.as_coeffs();
        current.push((
            "transform",
            format!(
                "matrix({} {} {} {} {} {})",
                fmt_num(a, precision),
                fmt_num(b, precision),
                fmt_num(c, precision),
                fmt_num(d, precision),
                fmt_num(e, precision),
                fmt_num(f, precision),
            ),
        ));
        current.push(("vector-effect", "non-scaling-stroke".to_string()))
    }
}
impl ToAttributes for Attr {
    fn add_as_attr(&self, current: &mut Vec<Attribute>, precision: usize) {
        match self {
            Attr::Fill(color) => current.push(("fill", color_to_svg(*color, precision))),
            Attr::StrokeColor(color) => current.push(("stroke", color_to_svg(*color, precision))),
            Attr::StrokeWidth(width) => current.push(("stroke-width", fmt_num(*width, precision))),
            Attr::StrokeJoin(join) => {
                current.push(("stroke-linejoin", stroke_join_to_svg(*join).to_string()))
            }
            Attr::StrokeCap(cap) => {
                current.push(("stroke-linecap", stroke_cap_to_svg(*cap).to_string()))
            }
            Attr::Opacity(opacity) => current.push(("opacity", fmt_num(*opacity, precision))),
            Attr::Id(id) => current.push(("id", id.clone())),
        }
    }
}

fn stroke_join_to_svg(join: StrokeJoin) -> &'static str {
    match join {
        StrokeJoin::Miter => "miter",
        StrokeJoin::Round => "round",
        StrokeJoin::Bevel => "bevel",
    }
}

fn stroke_cap_to_svg(cap: StrokeCap) -> &'static str {
    match cap {
        StrokeCap::Butt => "butt",
        StrokeCap::Round => "round",
        StrokeCap::Square => "square",
    }
}

/// Converts a color to an SVG color string
fn color_to_svg(color: Color, precision: usize) -> String {
    match color {
        Color::Rgba { r, g, b, a } => format!(
            "rgba({},{},{},{})",
            r.round().clamp(0.0, 255.0) as u8,
            g.round().clamp(0.0, 255.0) as u8,
            b.round().clamp(0.0, 255.0) as u8,
            fmt_num(a.clamp(0.0, 1.0), precision)
        ),
        Color::Hsla { h, s, l, a } => format!(
            "hsla({},{}%,{}%,{})",
            h.round().clamp(0.0, 360.0) as u16,
            s.round().clamp(0.0, 100.0) as u8,
            l.round().clamp(0.0, 100.0) as u8,
            fmt_num(a.clamp(0.0, 1.0), precision)
        ),
    }
}
