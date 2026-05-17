//! Abstract Syntax Tree nodes

use std::ops::Range;

use crate::ir::Number;

// --- Span ---

/// Source byte range. Captured from `LocatingSlice` during parsing.
pub type Span = Range<usize>;

/// Wraps a node with its source span.
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

pub type SpanExpr = Spanned<Expr>;
/// Boxed expression with span info.
pub type BoxExpr = Box<SpanExpr>;

// --- Primitives ---

/// Color symbol: in various representations
#[derive(Debug, Clone)]
pub enum Color {
    Rgba {
        r: BoxExpr,
        g: BoxExpr,
        b: BoxExpr,
        a: BoxExpr,
    },
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
    pub params: Vec<String>,
    pub scope: Vec<(String, SpanExpr)>,
    pub return_expr: BoxExpr,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(Number),
    String(String),
    Bool(bool),
    Color(Color),
    List(ListLiteral),
    Tuple(Vec<SpanExpr>),
}

/// Expression
#[derive(Debug, Clone)]
pub enum Expr {
    // Literals
    Literal(Literal),
    // Variable
    Variable(String),
    // Field access
    Field {
        object: BoxExpr,
        field: String,
    },
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
    Sin,
    Cos,
    Tan,
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
    // Graphic constructors
    Circle,
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
    Stroke,
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
}

#[derive(Debug, Clone)]
pub enum ProgramUnit {
    Setting(Setting),
    Assignment { identifier: String, value: SpanExpr },
    Function { identifier: String, func: Function },
    Export(SpanExpr),
    ExportEach(SpanExpr),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub units: Vec<Spanned<ProgramUnit>>,
}
