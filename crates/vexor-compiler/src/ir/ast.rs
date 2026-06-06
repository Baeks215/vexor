//! Abstract Syntax Tree nodes

use std::ops::Range;
use std::rc::Rc;

use crate::ir::{Ident, Number};

// --- Span ---

/// Source byte range. Captured from `LocatingSlice` during parsing.
pub type Span = Range<usize>;

/// Wraps a node with its source span.
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Option<Span>,
}

pub type SpanExpr = Spanned<Expr>;
/// Boxed expression with span info.
pub type BoxExpr = Box<SpanExpr>;

// Terse error construction for `Spanned<String>` (used as `EError`).
// Placeholder span `0..0` is overwritten by the eval funnel when bubbling out.
impl From<String> for Spanned<String> {
    fn from(node: String) -> Self {
        Spanned { node, span: None }
    }
}
impl From<&str> for Spanned<String> {
    fn from(s: &str) -> Self {
        Spanned {
            node: s.to_string(),
            span: None,
        }
    }
}

// --- Expressions ---

pub mod op {
    #[derive(Debug, Clone, Copy)]
    pub enum Binary {
        Arithmetic(Arithmetic),
        Compare(Compare),
        Logic(Logic),
        Cons,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Arithmetic {
        Add,
        Sub,
        Mul,
        Div,
        IntDiv,
        Rem,
        Pow,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Compare {
        Gt,
        Gte,
        Lt,
        Lte,
        Eq,
        Neq,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Logic {
        And,
        Or,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Unary {
        Not,
        Neg,
    }
}

#[derive(Debug, Clone)]
pub enum ListLiteral {
    List(Vec<SpanExpr>),
    Range {
        start: BoxExpr,
        second: Option<BoxExpr>,
        end: BoxExpr,
    },
}

#[derive(Debug, Clone)]
pub struct Function {
    /// Parameter identifiers of the function.
    pub params: Vec<Ident>,
    /// Where scope of identifier-expression bindings. Uses `Rc` to prevent deep clone per call.
    pub where_scope: Vec<(Ident, Rc<SpanExpr>)>,
    /// Return expression of the function.
    pub return_expr: BoxExpr,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(Number),
    String(String),
    Bool(bool),
    List(ListLiteral),
    Tuple(Vec<SpanExpr>),
}

/// Expression
#[derive(Debug, Clone)]
pub enum Expr {
    // Literals
    Literal(Literal),
    // Variable
    Variable(Ident),
    // Expressions with operators
    Binary {
        operator: op::Binary,
        left: BoxExpr,
        right: BoxExpr,
    },
    Unary {
        operator: op::Unary,
        operand: BoxExpr,
    },
    /// Function call
    Call {
        function: BoxExpr,
        args: Vec<SpanExpr>,
    },
    /// Anonymous function
    Function(Function),
    /// Standard Function Reference
    Std(Std),
    /// Constant
    Const(Const),
    /// Match expression
    Match {
        scrutinee: BoxExpr,
        arms: Vec<MatchArm>,
    },
    /// If expression
    If {
        condition: BoxExpr,
        then_branch: BoxExpr,
        else_branch: BoxExpr,
    },
}

// --- Standard Functions ---

#[derive(Debug, Clone, Copy)]
pub enum Const {
    Pi,
}

#[derive(Debug, Clone, Copy)]
pub enum Std {
    // Trig functions
    Rad,
    Deg,
    Sin,
    Cos,
    Tan,
    // Math functions
    Sinh,
    Cosh,
    Tanh,
    Asinh,
    Acosh,
    Atanh,
    Asin,
    Acos,
    Atan,
    Atan2,
    Round,
    Floor,
    Ceil,
    Abs,
    Log,
    Exp,
    Max,
    Min,
    Clamp,
    // Vector
    Magnitude,
    Normalize,
    Dot,
    // List
    Map,
    Filter,
    Drop,
    Take,
    DropWhile,
    TakeWhile,
    Foldl,
    Foldr,
    Zip,
    ZipWith,
    FlatMap,
    Enumerate,
    Len,
    Reverse,
    Find,
    Sort,
    SortBy,
    Repeat,
    Nth,
    Head,
    Tail,
    Last,
    Init,
    IsEmpty,
    Sum,
    Product,
    Concat,
    // Tuple
    Fst,
    Snd,
    // Color constructors
    Rgb,
    Rgba,
    Hsl,
    Hsla,
    // Graphic constructors
    Circle,
    Ellipse,
    Rect,
    Text,
    Group,
    Line,
    Curve,
    Path,
    Sample,
    // Graphic functions
    Close,
    JumpTo,
    LineTo,
    CurveTo,
    Move,
    Scale,
    Rotate,
    MirrorX,
    MirrorY,
    Fill,
    StrokeWidth,
    StrokeColor,
    StrokeJoin,
    StrokeCap,
    Opacity,
    SetId,
}

// --- Match ---

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: SpanExpr,
    pub guard: Option<SpanExpr>,
    pub body: SpanExpr,
}

// --- Program ---

#[derive(Debug, Clone)]
pub enum Setting {
    Canvas { width: usize, height: usize },
    Precision(usize),
}

#[derive(Debug, Clone)]
pub enum ProgramUnit {
    Setting(Setting),
    Assignment { identifier: Ident, value: SpanExpr },
    Function { identifier: Ident, func: Function },
    Export(SpanExpr),
    ExportEach(SpanExpr),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub units: Vec<Spanned<ProgramUnit>>,
}
