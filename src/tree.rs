// src/build.rs
use pest::iterators::{Pair, Pairs};
use pest::{Parser, Span};
use crate::parser::{Rule, G};
use crate::ast::*;

pub fn parse_program(input: &str) -> Result<Program, pest::error::Error<Rule>> {
    let mut pairs = G::parse(Rule::program, input)?;
    let program = pairs.next().unwrap();
    build_program(program)
}

fn build_program(p: Pair<Rule>) -> Result<Program, pest::error::Error<Rule>> {
    let mut instrs = Vec::new();
    for el in p.into_inner() {
        match el.as_rule() {
            Rule::instr => instrs.push(build_instr(el)?),
            _ => {}
        }
    }
    Ok(Program { instrs })
}

fn build_instr(p: Pair<Rule>) -> Result<Instr, pest::error::Error<Rule>> {
    let span = p.as_span();
    let mut i = p.into_inner(); // one of: query | rule | fact (+ trailing dot)
    let node = i.next().unwrap();
    Ok(match node.as_rule() {
        Rule::fact => {
            let functor = build_functor(node.into_inner().next().unwrap());
            Instr::Fact { functor, span }
        }
        Rule::rule => {
            let mut j = node.into_inner();
            let head = build_term(j.next().unwrap());          // term
            let body = build_expr(j.next().unwrap());          // expr_with_cut
            Instr::Rule { head, body, span }
        }
        Rule::query => {
            let expr = build_expr(node.into_inner().next().unwrap()); // expr
            Instr::Query { body: expr, span }
        }
        _ => unreachable!(),
    })
}

fn build_expr(p: Pair<Rule>) -> Expr {
    match p.as_rule() {
        Rule::expr_with_cut | Rule::expr | Rule::basic_expr => {
            let inner = p.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::not_provable => {
                    let span = inner.as_span();
                    let term = build_term(inner.into_inner().next().unwrap());
                    Expr::NotProvable(term, span)
                }
                Rule::expr_equal_bind => {
                    let span = inner.as_span();
                    let mut i = inner.into_inner();
                    let l = build_term(i.next().unwrap());
                    let r = build_term(i.next().unwrap());
                    Expr::EqualOrBind(l, r, span)
                }
                Rule::expr_not_equal_bind => {
                    let span = inner.as_span();
                    let mut i = inner.into_inner();
                    let l = build_term(i.next().unwrap());
                    let r = build_term(i.next().unwrap());
                    Expr::NotEqualOrBind(l, r, span)
                }
                Rule::term => {
                    let sp = inner.as_span();
                    let t = build_term(inner);
                    Expr::Term(t, sp)
                }
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}

fn build_term(p: Pair<Rule>) -> Term {
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

fn build_functor(p: Pair<Rule>) -> Term {
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

fn build_list(p: Pair<Rule>) -> Term {
    let span = p.as_span();
    let mut i = p.into_inner();
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
