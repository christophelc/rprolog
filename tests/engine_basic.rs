use std::collections::HashMap;
use rprolog::{
    parse_program,
    ast::{Instr, Expr},
    engine::{Database, solve_expr},
};

pub fn run_first_query<'a>(
    input: &'a str
) -> Vec<std::collections::HashMap<String, rprolog::ast::Term<'a>>> {
    let program = parse_program(input).expect("parse");          // Program<'a>
    let db: Database<'a> = Database::from_program(&program);     // db possède ses clauses

    let q = program.instrs.iter().find_map(|i| match i {
        rprolog::ast::Instr::Query { body, .. } => Some(body),
        _ => None,
    }).expect("expected a query");

    // On renvoie des Subst **possédés** (HashMap<String, Term<'a>>), pas des &Subst
    solve_expr(&db, q)
}

#[test]
fn query_parent_john_y() {
    let src = r#"
        parent(john, X).
        ?- parent(john, Y).
    "#;
    let sols = run_first_query(src);
    assert!(!sols.is_empty());
    // There’s at least one solution; you can further inspect bindings if you expose them.
}

#[test]
fn query_mother_of_alice() {
    let src = r#"
        parent(mary, john).
        parent(mary, alice).
        mother_of_alice(M) :- parent(M, alice).
        ?- mother_of_alice(M).
    "#;

    let program = parse_program(src).expect("parse");
    let db = Database::from_program(&program);

    // find the mother_of_alice query
    let query = program.instrs.iter().find_map(|i| match i {
        Instr::Query { body, .. } => Some(body),
        _ => None
    }).unwrap();

    let sols = solve_expr(&db, query);
    assert_eq!(sols.len(), 1); // expect exactly one M
    // If you expose a helper to pretty-print or extract specific vars, assert M == mary.
}

#[test]
fn query_equal_and_notprovable_expr() {
    // Test '=’ and '\+' separately
    let src_eq = "?- X = john.";
    let sols_eq = run_first_query(src_eq);
    assert_eq!(sols_eq.len(), 1);

    let src_not = "?- \\+ X = john.";
    let sols_not = run_first_query(src_not);
    assert_eq!(sols_not.len(), 0); // succeeds once, no bindings
}

