//! Abstract Syntax Tree nodes

use crate::ir::Number;

// --- Primitives ---

/// Color symbol: in various representations
#[derive(Debug, Clone)]
pub enum Color {
    Rgba {
        r: Box<Expr>,
        g: Box<Expr>,
        b: Box<Expr>,
        a: Box<Expr>,
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
    }
}

#[derive(Debug, Clone)]
pub enum ListLiteral {
    List(Vec<Expr>),
    Range {
        start: Box<Expr>,
        second: Option<Box<Expr>>,
        end: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub struct Function {
    pub params: Vec<String>,
    pub scope: Vec<(String, Expr)>,
    pub return_expr: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(Number),
    String(String),
    Bool(bool),
    Color(Color),
    List(ListLiteral),
    Tuple(Vec<Expr>),
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
        object: Box<Expr>,
        field: String,
    },
    // Expressions with operators
    Binary {
        operator: op::Binary,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        operator: op::Unary,
        operand: Box<Expr>,
    },
    /// Function call
    Call {
        function: Box<Expr>,
        args: Vec<Expr>,
    },
    /// Anonymous function
    Function(Function),
    /// Standard Function Reference
    Std(Std),
    /// Constant
    Const(Const),
    /// Match expression
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    /// If expression
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
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
    // Graphic constructors
    Circle,
    Rect,
    Text,
    Group,
    Line,
    Curve,
    Path,
    // Graphic functions
    Close,
    JumpTo,
    LineTo,
    CurveTo,
    Move,
    Scale,
    Rotate,
    Fill,
    Stroke,
}

// --- Match ---

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Expr,
    pub guard: Option<Expr>,
    pub body: Expr,
}

// --- Program ---

#[derive(Debug, Clone)]
pub enum Setting {
    Canvas { width: usize, height: usize },
}

#[derive(Debug, Clone)]
pub enum ProgramUnit {
    Setting(Setting),
    Assignment { identifier: String, value: Expr },
    Function { identifier: String, func: Function },
    Export(Expr),
    ExportEach(Expr),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub units: Vec<ProgramUnit>,
}
