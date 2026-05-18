use winnow::error::{AddContext, ContextError, ErrMode};
use winnow::stream::Stream;

use crate::parser::Input;

/// Helper functions for creating StrContext values for error reporting.
pub mod str_ctx {
    use winnow::error::{StrContext, StrContextValue};

    pub fn desc(desc: &'static str) -> StrContext {
        StrContext::Expected(StrContextValue::Description(desc))
    }
    pub fn lit(literal: &'static str) -> StrContext {
        StrContext::Expected(StrContextValue::StringLiteral(literal))
    }
    pub fn lit_char(char_literal: char) -> StrContext {
        StrContext::Expected(StrContextValue::CharLiteral(char_literal))
    }
    pub fn label(label: &'static str) -> StrContext {
        StrContext::Label(label)
    }
}

pub struct CtxErrBuilder<'a, 'b> {
    input: &'b mut Input<'a>,
    /// Final Context error
    pub err: ErrMode<ContextError>,
}

impl<'a, 'b> CtxErrBuilder<'a, 'b> {
    pub fn new(input: &'b mut Input<'a>) -> Self {
        Self {
            input,
            err: ErrMode::Cut(ContextError::new()),
        }
    }
    pub fn from_checkpoint(
        input: &'b mut Input<'a>,
        checkpoint: &<Input<'a> as Stream>::Checkpoint,
    ) -> Self {
        input.reset(checkpoint);
        Self::new(input)
    }
    /// Add expected description context to the error.
    pub fn expected(self, desc: &'static str) -> Self {
        let Self { input, err } = self;
        let err = err.add_context(input, &input.checkpoint(), str_ctx::desc(desc));
        Self { input, err }
    }
    /// Add expected string literal context to the error.
    pub fn label(self, label: &'static str) -> Self {
        let Self { input, err } = self;
        let err = err.add_context(input, &input.checkpoint(), str_ctx::label(label));
        Self { input, err }
    }
}
