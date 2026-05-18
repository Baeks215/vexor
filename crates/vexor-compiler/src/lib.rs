mod evaluator;
mod exporter;
mod ir;
mod parser;

use ariadne::{Config, Label, Report, ReportKind, Source};
use std::ops::Range;
use winnow::LocatingSlice;
use winnow::error::{ContextError, ParseError};

use crate::ir::ast::Spanned;

pub use exporter::*;
pub use ir::scene::{Color, Graphic, GraphicType, Scene};

pub struct CError<'a> {
    pub src: &'a str,
    pub span: Range<usize>,
    pub msg: String,
    pub title: &'static str,
}

/// Result type for evaluation
pub type CResult<'a, O> = Result<O, CError<'a>>;

/// Compiles the given input string into [`Scene`] IR.
pub fn compile<'a>(input: &'a str) -> CResult<'a, Scene> {
    let ast = parser::parse_program(input).map_err(|e| build_parse_error(input, e))?;
    let scene = evaluator::eval_program(ast).map_err(|e| build_eval_error(input, e))?;
    Ok(scene)
}

/// Compiles input with given exporter
fn compile_to<'a, T>(
    input: &'a str,
    exporter: fn(Scene) -> Vec<Export<T>>,
) -> CResult<'a, Vec<Export<T>>> {
    let scene = compile(input)?;
    Ok(exporter(scene))
}

/// Compiles input to SVG exports
pub fn compile_to_svg<'a>(input: &'a str) -> CResult<'a, Vec<Export<String>>> {
    compile_to(input, export_scene_svg)
}

// --- Error rendering ---

fn build_parse_error<'a>(
    src: &'a str,
    e: ParseError<LocatingSlice<&str>, ContextError>,
) -> CError<'a> {
    let offset = e.offset();
    let end = (offset + 1).min(src.len()).max(offset);
    let span = offset..end;
    let msg = e.inner().to_string();
    CError {
        src,
        span,
        msg,
        title: "parse error",
    }
}

fn build_eval_error<'a>(src: &'a str, e: evaluator::EError) -> CError<'a> {
    let Spanned { node, span } = e;
    let span = span.unwrap_or(0..0);
    CError {
        src,
        span,
        msg: node,
        title: "evaluation error",
    }
}

impl<'a> CError<'a> {
    const SOURCE_ID: &'static str = "input";

    /// Formats the error message with colors
    pub fn format_colored(&self) -> String {
        self.render(true)
    }

    fn render(&self, color: bool) -> String {
        let Self {
            src,
            span,
            msg,
            title,
        } = self;
        let mut buf: Vec<u8> = Vec::new();
        let mut report = Report::build(ReportKind::Error, (Self::SOURCE_ID, span.clone()))
            .with_config(Config::new().with_color(color))
            .with_message(title);
        for line in msg.lines() {
            report = report.with_label(
                Label::new((Self::SOURCE_ID, span.clone()))
                    .with_message(line)
                    .with_color(ariadne::Color::Red),
            );
        }

        report
            .finish()
            .write((Self::SOURCE_ID, Source::from(src)), &mut buf)
            .ok();
        String::from_utf8(buf).unwrap_or_default()
    }
}
impl<'a> std::fmt::Display for CError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Format with no color by default
        write!(f, "{}", self.render(false))
    }
}
impl<'a> std::fmt::Debug for CError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.render(false))
    }
}
