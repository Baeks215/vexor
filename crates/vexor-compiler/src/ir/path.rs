//! Path representation and manipulation.

use im_rc::Vector;
use kurbo::{Affine, PathEl, Point};

use crate::evaluator::EResult;
use crate::exporter::fmt_num;
use crate::ir::scene::{Graphic, GraphicType};

/// A vector path, internally a persistent vector of path elements.
#[derive(Debug, Clone, Default)]
pub struct Path {
    els: Vector<PathEl>,
    /// Number of elements that are not `MoveTo` (line/curve/close).
    non_move: usize,
}

impl Path {
    /// Creates an empty path.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if the path has a drawn element and can therefore be closed.
    pub fn can_close(&self) -> bool {
        self.non_move > 0
    }

    /// Appends a move-to element, starting a new subpath at `p`.
    pub fn move_to(&mut self, p: Point) {
        self.els.push_back(PathEl::MoveTo(p));
    }

    /// Appends a line-to element from the current point to `p`.
    pub fn line_to(&mut self, p: Point) {
        self.els.push_back(PathEl::LineTo(p));
        self.non_move += 1;
    }

    /// Appends a cubic Bézier curve through control points `p1`, `p2` to `p3`.
    pub fn curve_to(&mut self, p1: Point, p2: Point, p3: Point) {
        self.els.push_back(PathEl::CurveTo(p1, p2, p3));
        self.non_move += 1;
    }

    /// Appends a close-path element.
    pub fn close_path(&mut self) {
        self.els.push_back(PathEl::ClosePath);
        self.non_move += 1;
    }

    /// Applies an affine transformation to every element of the path.
    pub fn apply_affine(&mut self, a: Affine) {
        // Element kinds are preserved, so `non_move` is unchanged.
        self.els = self.els.iter().map(|el| a * *el).collect();
    }

    /// Concatenates `other` onto the end of this path.
    ///   Persistent vector append is `O(log n)`.
    pub fn append(&mut self, other: Path) {
        self.els.append(other.els);
        self.non_move += other.non_move;
    }

    /// Iterates over the path elements.
    pub fn iter(&self) -> impl Iterator<Item = &PathEl> {
        self.els.iter()
    }

    /// Converts the path to an SVG path data string
    pub fn to_svg(&self, precision: usize) -> String {
        let pt = |p: Point| format!("{},{}", fmt_num(p.x, precision), fmt_num(p.y, precision));
        let mut out = String::new();
        // Path data must start with a MoveTo, so prepend an origin one if missing.
        if !matches!(self.els.front(), Some(PathEl::MoveTo(_))) {
            out.push_str(&format!("M{}", pt(Point::ORIGIN)));
        }
        for el in &self.els {
            if !out.is_empty() {
                out.push(' ');
            }
            match *el {
                PathEl::MoveTo(p) => out.push_str(&format!("M{}", pt(p))),
                PathEl::LineTo(p) => out.push_str(&format!("L{}", pt(p))),
                PathEl::QuadTo(p1, p2) => out.push_str(&format!("Q{} {}", pt(p1), pt(p2))),
                PathEl::CurveTo(p1, p2, p3) => {
                    out.push_str(&format!("C{} {} {}", pt(p1), pt(p2), pt(p3)))
                }
                PathEl::ClosePath => out.push('Z'),
            }
        }
        out
    }
}

impl IntoIterator for Path {
    type Item = PathEl;
    type IntoIter = <Vector<PathEl> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.els.into_iter()
    }
}

/// Applies an in-place transformation to the path of the graphic component, if it is a path.
pub fn transform_path(g: Graphic, f: impl FnOnce(&mut Path) -> EResult<()>) -> EResult<Graphic> {
    let Graphic {
        ty,
        attr,
        transform,
    } = g;
    let mut path = match ty {
        GraphicType::Path { path } => path,
        _ => return Err("expected a path".into()),
    };
    f(&mut path)?;
    Ok(Graphic {
        ty: GraphicType::Path { path },
        attr,
        transform,
    })
}

/// Closes a path.
pub fn close_path(path: &mut Path) -> EResult<()> {
    if !path.can_close() {
        return Err("cannot close an empty path".into());
    }
    path.close_path();
    Ok(())
}

/// Concatenates two paths:
///   Translates `right` so its start meets `left`'s end, and connects the two paths.
pub fn concat_paths(left: &mut Path, mut right: Path) -> EResult<()> {
    let end = path_end(left).ok_or("cannot concatenate: left path is empty")?;
    let start = match right.els.front() {
        Some(PathEl::MoveTo(p)) => *p,
        _ => Point::ORIGIN,
    };
    let delta = end - start;
    right.apply_affine(Affine::translate(delta));
    left.append(right);
    Ok(())
}

/// Builds a smooth cubic path through every point in `points` via
/// Catmull-Rom → Bézier conversion.
///   Phantom endpoints are synthesized internally by linearly extrapolating each
///   end segment (reflecting it outward), so the curve passes through all `points`.
pub fn catmull_rom_path(points: &[Point]) -> Path {
    let mut path = Path::new();
    if points.len() < 2 {
        return path;
    }
    let n = points.len();
    // Reflect each end segment outward by extrapolation
    let start = points[0] + (points[0] - points[1]);
    let end = points[n - 1] + (points[n - 1] - points[n - 2]);
    let pts: Vec<Point> = std::iter::once(start)
        .chain(points.iter().copied())
        .chain(std::iter::once(end))
        .collect();

    // Translate so the first drawn point starts at the origin.
    let offset = Point::ORIGIN - pts[1];
    for i in 1..pts.len() - 2 {
        let p0 = pts[i - 1];
        let p1 = pts[i];
        let p2 = pts[i + 1];
        let p3 = pts[i + 2];
        let b1 = p1 + (p2 - p0) / 6.0;
        let b2 = p2 - (p3 - p1) / 6.0;
        path.curve_to(b1 + offset, b2 + offset, p2 + offset);
    }
    path
}

/// Returns the endpoint of a path, or `None`
fn path_end(path: &Path) -> Option<Point> {
    let last = path.els.back()?;
    match *last {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => Some(p),
        PathEl::QuadTo(_, p) => Some(p),
        PathEl::CurveTo(_, _, p) => Some(p),
        PathEl::ClosePath => None,
    }
}
