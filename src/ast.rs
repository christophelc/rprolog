// src/ast.rs
use pest::Span;

#[derive(Debug, Clone)]
pub struct Program<'a> {
    pub instrs: Vec<Instr<'a>>,
}

#[derive(Debug, Clone)]
pub enum Instr<'a> {
    Fact { functor: Term<'a>, span: Span<'a> },
    Rule { head: Term<'a>, body: Expr<'a>, span: Span<'a> },
    Query { body: Expr<'a>, span: Span<'a> },
}

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    // For now your grammar has single-term / (=)/(\=)/\+ cases.
    // If you later add ',' and ';' precedence, add And/Or nodes here.
    Term(Term<'a>, Span<'a>),
    EqualOrBind(Term<'a>, Term<'a>, Span<'a>),    // =
    NotEqualOrBind(Term<'a>, Term<'a>, Span<'a>), // \=
    NotProvable(Term<'a>, Span<'a>),              // \+ Goal
    Cut(Span<'a>),                                // if you allow ! inside expr
}

#[derive(Debug, Clone)]
pub enum Term<'a> {
    Atom(String, Span<'a>),                 // StringValue / SymbolAtom
    StringLit(String, Span<'a>),            // if you want string vs atom separate
    Number(String, Span<'a>),               // parse BigDecimal later
    Variable(String, Span<'a>),             // UntypedVariable
    Functor { name: String, args: Vec<Term<'a>>, span: Span<'a> },
    ListEmpty(Span<'a>),
    ListCons(Box<Term<'a>>, Box<Term<'a>>, Span<'a>), // NonEmptyList(head, tail)
    Cut(Span<'a>),
}
