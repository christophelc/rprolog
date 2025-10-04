use pest::Parser;

use rprolog::parser::Rule;
use rprolog::engine::{Database, solve_expr, Subst};
use rprolog::ast::{Instr, Program};
use rprolog::tree::{build_program, parse_program, print_pair};

fn exec_prog(prog: &Program) {
    // 2) DB from facts/rules
    let db = Database::from_program(prog);

    // 3) run queries found in the program
    for ins in &prog.instrs {
        if let Instr::Query { body, .. } = ins {
            println!("?- {:?}\n  answers:", body);
            let sols = solve_expr(&db, body);
            if sols.is_empty() {
                println!("  no.");
            } else {
                for s in sols {
                    println!("  {}", pretty_subst(&s));
                }
            }
        }
    }
}

fn main() {
    let input = r#"
        parent(john, X).
        parent(mary, john).
        parent(mary, alice).
        % A rule example (works because body is a single goal here):
        mother_of_alice(M) :- parent(M, alice).
        ?- parent(john, Y).
        ?- mother_of_alice(M).
        ?- X = john.
        ?- \+ X = john.
    "#;
    let result: Result<pest::iterators::Pairs<'_, Rule>, pest::error::Error<Rule>> = rprolog::parser::G::parse(rprolog::parser::Rule::program, input);    
    if let Ok(pairs) = result {
        for pair in pairs {
            print_pair(&pair, 0);
        }
         if let Ok(program) = parse_program(input) {
            println!("Parsed program: {program:#?}");
            exec_prog(&program);
        } else {
            eprintln!("Failed to build program from parse tree.");
        }
    }
    else {
        println!("Failed to parse input: {:?}", result);
    }
}

fn pretty_subst<'a>(s: &Subst<'a>) -> String {
    if s.is_empty() { "yes.".to_string() }
    else {
        s.iter().map(|(k,v)| format!("{k} = {v:?}")).collect::<Vec<_>>().join(", ")
    }
}

