// tests/parser_rules.rs
use rprolog::{parse_program, ast::{Instr, Term}};

#[test]
fn parses_mother_of_alice_rule() {
    let src = r#"
        mother_of_alice(M) :- parent(M, alice).
    "#;

    let program = parse_program(src).expect("parse program");
    assert_eq!(program.instrs.len(), 1);

    match &program.instrs[0] {
        Instr::Rule { head, body, .. } => {
            // head should be mother_of_alice(M)
            match head {
                Term::Functor { name, args, .. } => {
                    assert_eq!(name, "mother_of_alice");
                    assert_eq!(args.len(), 1);
                }
                _ => panic!("head should be functor"),
            }

            // body should be Expr::Term(Functor(parent, [M, alice]))
            match body {
                rprolog::ast::Expr::Term(t, _) => {
                    match t {
                        Term::Functor { name, args, .. } => {
                            assert_eq!(name, "parent");
                            assert_eq!(args.len(), 2);
                        }
                        _ => panic!("body term should be functor"),
                    }
                }
                other => panic!("unexpected body expr: {:?}", other),
            }
        }
        other => panic!("expected rule, got {:?}", other),
    }
}
