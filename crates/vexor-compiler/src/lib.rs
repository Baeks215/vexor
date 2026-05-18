mod evaluator;
mod exporter;
mod ir;
mod parser;

use ariadne::{Config, Label, Report, ReportKind, Source};
use std::ops::Range;
use winnow::LocatingSlice;
use winnow::error::{ContextError, ParseError};

pub use exporter::*;
pub use ir::scene::{Color, Graphic, GraphicType, Scene};

/// Result type for evaluation
pub type CResult<O> = Result<O, String>;

const SOURCE_ID: &str = "input";

/// Compiles the given input string into [`Scene`] IR.
pub fn compile(input: &str) -> CResult<Scene> {
    let ast = parser::parse_program(input).map_err(|e| render_parse_error(input, &e))?;
    let scene = evaluator::eval_program(ast).map_err(|e| render_eval_error(input, &e))?;
    Ok(scene)
}

/// Compiles input with given exporter
fn compile_to<T>(input: &str, exporter: fn(Scene) -> Vec<Export<T>>) -> CResult<Vec<Export<T>>> {
    let scene = compile(input)?;
    Ok(exporter(scene))
}

/// Compiles input to SVG exports
pub fn compile_to_svg(input: &str) -> CResult<Vec<Export<String>>> {
    compile_to(input, export_scene_svg)
}

// --- Error rendering ---

fn render_parse_error(src: &str, e: &ParseError<LocatingSlice<&str>, ContextError>) -> String {
    let offset = e.offset();
    let end = (offset + 1).min(src.len()).max(offset);
    let span = offset..end;
    let msg = e.inner().to_string();
    render_report(src, span, &msg, "parse error")
}

fn render_eval_error(src: &str, e: &evaluator::EError) -> String {
    let span = e.span.clone().unwrap_or(0..0);
    render_report(src, span, &e.node, "evaluation error")
}

fn render_report(src: &str, span: Range<usize>, msg: &str, label: &str) -> String {
    let mut buf: Vec<u8> = Vec::new();
    let mut report = Report::build(ReportKind::Error, (SOURCE_ID, span.clone()))
        .with_config(Config::new().with_color(false))
        .with_message(label);
    for line in msg.lines() {
        report = report.with_label(Label::new((SOURCE_ID, span.clone())).with_message(line));
    }

    report
        .finish()
        .write((SOURCE_ID, Source::from(src)), &mut buf)
        .ok();
    String::from_utf8(buf).unwrap_or_default()
}
