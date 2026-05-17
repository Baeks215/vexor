//! Graphic utilities for the evaluator.

use kurbo::{Affine, BezPath, PathEl, Point};

use crate::evaluator::EResult;
use crate::ir::scene::{Graphic, GraphicType};

/// Applies a transformation to the path of the graphic component, if it is a path.
pub fn transform_path(g: Graphic, f: impl FnOnce(BezPath) -> EResult<BezPath>) -> EResult<Graphic> {
    let Graphic {
        ty,
        style,
        transform,
    } = g;
    let path = match ty {
        GraphicType::Path { path } => f(path)?,
        _ => return Err("expected a path".to_string()),
    };
    Ok(Graphic {
        ty: GraphicType::Path { path },
        style,
        transform,
    })
}

/// Creates a new path starting at the origin.
pub fn start_path() -> BezPath {
    let mut path = BezPath::new();
    path.move_to(Point::ORIGIN);
    path
}

/// Closes a path.
pub fn close_path(mut path: BezPath) -> EResult<BezPath> {
    if path.is_empty() {
        return Err("cannot close an empty path".into());
    }
    path.close_path();
    Ok(path)
}

/// Normalise path to start at the origin.
pub fn normalise_path(mut path: BezPath) -> BezPath {
    let Some(PathEl::MoveTo(first)) = path.elements().first() else {
        return path;
    };
    let offset = Point::ORIGIN - *first;
    path.apply_affine(Affine::translate(offset));
    path
}

/// Concatenates two paths:
///   Translates `right` so its start meets `left`'s end, and connects the two paths.
pub fn concat_paths(mut left: BezPath, mut right: BezPath) -> EResult<BezPath> {
    let end = path_end(&left).ok_or("cannot concatenate: left path is empty")?;
    let mut moved = false;
    let start = match right.elements().first() {
        Some(PathEl::MoveTo(p)) => {
            moved = true;
            *p
        }
        _ => Point::ORIGIN,
    };
    let delta = end - start;
    right.apply_affine(Affine::translate(delta));
    left.extend(right.into_iter().skip(if moved { 1 } else { 0 }));
    Ok(left)
}

/// Builds a smooth cubic BezPath through `points` via uniform Catmull-Rom → Bézier conversion.
///   Start and end points should be phantom points,
///   i.e. Not included in the path but used to determine tangents at the endpoints.
pub fn catmull_rom_path(points: &[Point]) -> BezPath {
    let mut path = BezPath::new();
    if points.len() < 2 {
        return path;
    }
    path.move_to(points[1]);
    for i in 1..points.len() - 2 {
        let p0 = points[i - 1];
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = points[i + 2];
        let b1 = p1 + (p2 - p0) / 6.0;
        let b2 = p2 - (p3 - p1) / 6.0;
        path.curve_to(b1, b2, p2);
    }
    normalise_path(path)
}

/// Returns the endpoint of a path, or `None`
fn path_end(path: &BezPath) -> Option<Point> {
    let els = path.elements();
    let last = els.last()?;
    match *last {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => Some(p),
        PathEl::QuadTo(_, p) => Some(p),
        PathEl::CurveTo(_, _, p) => Some(p),
        PathEl::ClosePath => None,
    }
}
