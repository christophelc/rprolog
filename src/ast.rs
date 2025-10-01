#[derive(Debug, Clone)]
pub enum Term {
    Variable(String),
    Number(f64),
    Atom(String),
    Functor { name: String, args: Vec<Term> },
    List(Vec<Term>),
    // ... add more as needed
}

#[derive(Debug, Clone)]
pub enum Expr {
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    EqualOrBind(Box<Term>, Box<Term>),
    NotEqualOrBind(Box<Term>, Box<Term>),
    Cut,
    // ... add more as needed
}

#[derive(Debug, Clone)]
pub enum Predicate {
    Fact(Term),
    Rule(Term, Expr),
    Query(Expr),
}