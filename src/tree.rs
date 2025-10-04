use pest::iterators::Pair;
use pest::{Parser, Span};
use crate::parser::{Rule, G};
use crate::ast::*;

pub fn print_pair(pair: &pest::iterators::Pair<Rule>, indent: usize) {
    let rule = format!("{:?}", pair.as_rule());
    let _span = pair.as_span();
    let text = pair.as_str().trim();
    let pad = "  ".repeat(indent);
    if pair.clone().into_inner().peekable().peek().is_some() {
        println!("{pad}{rule}:");
        for inner in pair.clone().into_inner() {
            print_pair(&inner, indent + 1);
        }
    } else {
        println!("{pad}{rule}: \"{text}\"");
    }
}

pub fn parse_program(input: &'_ str) -> Result<Program<'_>, pest::error::Error<Rule>> {
    let mut pairs = G::parse(Rule::program, input)?;
    let program = pairs.next().unwrap();
    build_program(program)
}

pub fn build_program(p: Pair<Rule>) -> Result<Program, pest::error::Error<Rule>> {
    let mut instrs = Vec::new();
    for el in p.into_inner() {
        match el.as_rule() {
            Rule::fact | Rule::rule | Rule::query => {
                let built = build_instr(el.clone())?;
                //println!("build_instr: {:?}", built);
                instrs.push(built);
            }
            Rule::EOI => { /* ignore */ }
            other => {
                println!("Skipping unexpected: {:?}", other);
            }
        }
    }
    Ok(Program { instrs })
}

fn build_instr(p: Pair<Rule>) -> Result<Instr, pest::error::Error<Rule>> {
    match p.as_rule() {
        Rule::fact  => build_fact(p),
        Rule::rule  => build_rule(p),
        Rule::query => build_query(p),
        _ => unreachable!("build_instr received unexpected: {:?}", p.as_rule()),
    }
}

fn build_fact(p: Pair<Rule>) -> Result<Instr, pest::error::Error<Rule>> {
    // fact = { functor ~ dot }
    let span = p.as_span();
    let mut it = p.into_inner();
    let functor_pair = it.next().expect("fact should start with functor");
    debug_assert_eq!(functor_pair.as_rule(), Rule::functor);
    // next is dot, which we can ignore:
    // let _dot = it.next();
    let functor = build_functor(functor_pair);
    Ok(Instr::Fact { functor, span })
}

fn build_rule(p: Pair<Rule>) -> Result<Instr, pest::error::Error<Rule>> {
    // rule = { term ~ implied_by ~ expr_with_cut ~ dot }
    let span = p.as_span();
    let mut it = p.into_inner();

    // term is silent => first visible child is its inner node (functor/list/variable/...)
    let head = build_term(it.next().expect("rule head term missing"));

    // implied_by & dot are silent => next visible child is the body (direct leaf)
    let body_pair = it.next().expect("rule body expr missing");
    let body = build_expr(body_pair);

    Ok(Instr::Rule { head, body, span })
}

fn build_query(p: Pair<Rule>) -> Result<Instr, pest::error::Error<Rule>> {
    let span = p.as_span();

    // pick the first child that can form an expression
    let body_node = p.into_inner()
        .find(|ch| matches!(ch.as_rule(),
            Rule::expr_with_cut | Rule::expr | Rule::basic_expr |
            Rule::not_provable | Rule::expr_equal_bind | Rule::expr_not_equal_bind |
            Rule::term | Rule::functor | Rule::list | Rule::atom | Rule::number | Rule::variable | Rule::cut
        ))
        .expect("query body missing");

    let body = build_expr(body_node);
    Ok(Instr::Query { body, span })
}


fn build_expr<'a>(p: Pair<'a, Rule>) -> Expr<'a> {
    // 1) unwrap any silent wrapper levels (expr_with_cut, expr, basic_expr)
    let mut node = p;
    loop {
        match node.as_rule() {
            Rule::expr_with_cut | Rule::expr | Rule::basic_expr => {
                node = node.into_inner().next().unwrap();
            }
            _ => break, // reached a real leaf node
        }
    }

    // 2) build from the leaf
    match node.as_rule() {
        Rule::cut => Expr::Cut(node.clone().as_span()),

        Rule::not_provable => {
            // Your grammar: not_provable = { "\\+" ~ expr }
            // Your AST: NotProvable(Term, Span)
            // -> If expr after \+ isn't a Term, we currently reject it.
            let inner = node.clone().into_inner().next().unwrap();
            let inner_expr = build_expr(inner);
            Expr::NotProvableExpr(Box::new(inner_expr), node.as_span())            
        }

        Rule::expr_equal_bind => {
            let span = node.as_span();
            let mut i = node.into_inner();
            let l = build_term(i.next().unwrap());
            let r = build_term(i.next().unwrap());
            Expr::EqualOrBind(l, r, span)
        }

        Rule::expr_not_equal_bind => {
            let span = node.as_span();
            let mut i = node.into_inner();
            let l = build_term(i.next().unwrap());
            let r = build_term(i.next().unwrap());
            Expr::NotEqualOrBind(l, r, span)
        }
        Rule::term
                | Rule::functor
                | Rule::list
                | Rule::atom
                | Rule::number
                | Rule::variable => {
                    let sp = node.as_span();
                    let t = build_term(node);
                    Expr::Term(t, sp)
                }
        other => unreachable!("unexpected node in build_expr: {:?}", other),
    }
}


fn build_term<'a>(p: Pair<'a, Rule>) -> Term<'a> {
    match p.as_rule() {
        Rule::simple_term => build_term(p.into_inner().next().unwrap()),
        Rule::complex_term => build_term(p.into_inner().next().unwrap()),
        Rule::constant => build_term(p.into_inner().next().unwrap()),
        Rule::variable => Term::Variable(p.as_str().to_string(), p.as_span()),
        Rule::number => Term::Number(p.as_str().to_string(), p.as_span()),
        Rule::atom => {
            // Your atom can be single-quoted, bare, or string.
            // If you want string literals distinct from atoms:
            let inner = p.clone().into_inner().next().unwrap_or(p.clone());
            match inner.as_rule() {
                Rule::string => Term::StringLit(unquote_double(inner.as_str()), p.as_span()),
                Rule::symbol_in_single_quote => Term::Atom(unquote_single(inner.as_str()), p.as_span()),
                Rule::symbol_without_single_quote => Term::Atom(inner.as_str().to_string(), p.as_span()),
                _ => Term::Atom(p.as_str().to_string(), p.as_span()),
            }
        }
        Rule::functor => build_functor(p),
        Rule::list => build_list(p),
        Rule::cut_backtracking => Term::Cut(p.as_span()),
        Rule::term => build_term(p.into_inner().next().unwrap()),
        _ => unreachable!("{:?}", p.as_rule()),
    }
}

fn build_functor<'a>(p: Pair<'a, Rule>) -> Term<'a> {
    let span = p.as_span();
    let mut i = p.into_inner();
    let name_pair = i.next().unwrap(); // atom
    let name = match name_pair.as_rule() {
        Rule::atom => {
            let inner = name_pair.clone().into_inner().next().unwrap_or(name_pair);
            match inner.as_rule() {
                Rule::string => unquote_double(inner.as_str()),
                Rule::symbol_in_single_quote => unquote_single(inner.as_str()),
                _ => inner.as_str().to_string(),
            }
        }
        _ => name_pair.as_str().to_string(),
    };
    let args = match i.next() {
        Some(list) if list.as_rule() == Rule::term_list => {
            list.into_inner().map(build_term).collect()
        }
        _ => Vec::new(),
    };
    Term::Functor { name, args, span }
}

fn build_list<'a>(p: Pair<'a, Rule>) -> Term<'a> {
    let span = p.as_span();
    let i = p.into_inner();
    // three shapes: [] | [a,b] | [H|T]
    if i.len() == 0 {
        return Term::ListEmpty(span);
    }
    // Detect split_list form
    let v: Vec<_> = i.collect();
    if v.len() >= 2 && v.iter().any(|pp| pp.as_rule() == Rule::split_list) {
        // [Head, ..., LastHead | Tail]
        // For Prolog semantics, convert [a,b|T] → cons(a, cons(b, T))
        let mut head_terms: Vec<Term> = Vec::new();
        let mut tail_term: Option<Term> = None;
        let mut before_bar = true;
        for pp in v {
            match pp.as_rule() {
                Rule::term_list if before_bar => head_terms.extend(pp.into_inner().map(build_term)),
                Rule::split_list => before_bar = false,
                Rule::term if !before_bar => tail_term = Some(build_term(pp)),
                _ => {}
            }
        }
        let mut list = tail_term.unwrap();
        for t in head_terms.into_iter().rev() {
            list = Term::ListCons(Box::new(t), Box::new(list), span.clone());
        }
        list
    } else {
        // [a,b,c] → cons(a, cons(b, cons(c, [])))
        let elems: Vec<Term> = v
            .into_iter()
            .flat_map(|pp| if pp.as_rule() == Rule::term_list {
                pp.into_inner().map(build_term).collect::<Vec<_>>()
            } else { vec![] })
            .collect();
        let mut list = Term::ListEmpty(span.clone());
        for t in elems.into_iter().rev() {
            list = Term::ListCons(Box::new(t), Box::new(list), span.clone());
        }
        list
    }
}

fn term_span<'a>(t: &'a Term<'a>) -> Span<'a> {
    match t {
        Term::Atom(_, s)
        | Term::StringLit(_, s)
        | Term::Number(_, s)
        | Term::Variable(_, s)
        | Term::Functor { span: s, .. }
        | Term::ListEmpty(s)
        | Term::ListCons(_, _, s)
        | Term::Cut(s) => s.clone(),
    }
}

fn unquote_single(s: &str) -> String {
    s.trim_start_matches('\'')
        .trim_end_matches('\'')
        .replace("\\'", "'")
}
fn unquote_double(s: &str) -> String {
    s.trim_start_matches('"')
        .trim_end_matches('"')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{G, Rule};
    use pest::Parser;

    #[test]
    fn parse_single_fact() {
        let src = "parent(john, X).";
        let mut pairs = G::parse(Rule::program, src).unwrap();
        let program = build_program(pairs.next().unwrap()).unwrap();
        assert_eq!(program.instrs.len(), 1);
        match &program.instrs[0] {
            crate::ast::Instr::Fact { functor, .. } => {
                // quick smoke checks
                if let crate::ast::Term::Functor { name, args, .. } = functor {
                    assert_eq!(name, "parent");
                    assert_eq!(args.len(), 2);
                } else { panic!("expected functor"); }
            }
            _ => panic!("expected fact"),
        }
    }

    #[test]
    fn parse_single_query_equal() {
        let src = "?- X = john.";
        let mut pairs = G::parse(Rule::program, src).unwrap();
        let program = build_program(pairs.next().unwrap()).unwrap();
        assert_eq!(program.instrs.len(), 1);
        matches!(&program.instrs[0], crate::ast::Instr::Query { .. });
    }
}
