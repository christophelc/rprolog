use std::collections::HashMap;
use crate::ast::{Expr, Instr, Program, Term};

type Var = String;

// A substitution: variable name -> bound term
pub type Subst<'a> = HashMap<Var, Term<'a>>;

// Get representative (chase bindings)
fn deref<'a>(t: Term<'a>, s: &Subst<'a>) -> Term<'a> {
    match t {
        Term::Variable(ref v, _) if s.contains_key(v) => {
            let bound = s.get(v).unwrap().clone();
            deref(bound, s)
        }
        _ => t,
    }
}

fn occurs<'a>(v: &str, t: &Term<'a>, s: &Subst<'a>) -> bool {
    match deref(t.clone(), s) {
        Term::Variable(ref name, _) => name == v,
        Term::Functor { args, .. } => args.iter().any(|a| occurs(v, a, s)),
        Term::ListCons(ref h, ref tl, _) => occurs(v, h, s) || occurs(v, tl, s),
        _ => false,
    }
}

// Unify t1 and t2 under an existing substitution
pub fn unify<'a>(t1: &Term<'a>, t2: &Term<'a>, subst: &Subst<'a>) -> Option<Subst<'a>> {
    use Term::*;
    let t1 = deref(t1.clone(), subst);
    let t2 = deref(t2.clone(), subst);

    match (t1, t2) {
        (Variable(v, _), t) | (t, Variable(v, _)) => {
            // If t is exactly the same variable, succeed with no change
            if let Variable(w, _) = &t {
                if *w == v {
                    return Some(subst.clone());
                }
            }
            // Standard occurs-check (prevents X = f(X), but allows X = Y)
            if occurs(&v, &t, subst) { return None; }
            let mut out = subst.clone();
            out.insert(v, t);
            Some(out)
        }
        (Atom(a, _), Atom(b, _)) => (a == b).then(|| subst.clone()),
        (StringLit(a, _), StringLit(b, _)) => (a == b).then(|| subst.clone()),
        (Number(a, _), Number(b, _)) => (a == b).then(|| subst.clone()),
        (Cut(_), Cut(_)) => Some(subst.clone()),

        (Functor { name: n1, args: a1, .. }, Functor { name: n2, args: a2, .. }) => {
            if n1 != n2 || a1.len() != a2.len() { return None; }
            let mut s = subst.clone();
            for (x, y) in a1.iter().zip(a2.iter()) {
                s = unify(x, y, &s)?;
            }
            Some(s)
        }
        (ListEmpty(_), ListEmpty(_)) => Some(subst.clone()),
        (ListCons(h1, t1, _), ListCons(h2, t2, _)) => {
            let s1 = unify(&h1, &h2, subst)?;
            unify(&t1, &t2, &s1)
        }
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum Clause<'a> {
    Fact(Term<'a>),
    Rule { head: Term<'a>, body: Expr<'a> },
}

#[derive(Default, Debug)]
pub struct Database<'a> {
    pub clauses: Vec<Clause<'a>>,
}

impl<'a> Database<'a> {
    pub fn from_program(p: &Program<'a>) -> Self {
        let mut db = Database::default();
        for ins in &p.instrs {
            match ins {
                Instr::Fact { functor, .. } => db.clauses.push(Clause::Fact(functor.clone())),
                Instr::Rule { head, body, .. } => db.clauses.push(Clause::Rule {
                    head: head.clone(),
                    body: body.clone(),
                }),
                Instr::Query { .. } => { /* queries are executed, not stored */ }
            }
        }
        db
    }
}

pub fn solve_expr<'a>(db: &Database<'a>, goal: &Expr<'a>) -> Vec<Subst<'a>> {
    use std::collections::HashMap;

    match goal {
        Expr::Term(t, _) => solve_term(db, t, &HashMap::new()),

        Expr::EqualOrBind(l, r, _) => {
            unify(l, r, &HashMap::new()).into_iter().collect()
        }

        Expr::NotEqualOrBind(l, r, _) => {
            if unify(l, r, &HashMap::new()).is_some() { vec![] } else { vec![HashMap::new()] }
        }

        // NEW: \+ <expr> — “negation as failure”
        Expr::NotProvableExpr(inner, _) => {
            let sols = solve_expr(db, inner);
            if sols.is_empty() { vec![HashMap::new()] } else { vec![] }
        }

        // Placeholder semantics until you add conjunction/disjunction
        Expr::Cut(_) => vec![HashMap::new()],
    }
}


fn solve_term<'a>(db: &Database<'a>, goal: &Term<'a>, seed: &Subst<'a>) -> Vec<Subst<'a>> {
    use Clause::*;
    let mut out = Vec::new();

    for clause in &db.clauses {
        match clause {
            Fact(head) => {
                if let Some(s) = unify(head, goal, seed) {
                    out.push(s);
                }
            }
            Rule { head, body } => {
                if let Some(s1) = unify(head, goal, seed) {
                    // Since Expr is a single goal in your current grammar,
                    // just solve the body under s1.
                    for s2 in solve_expr(db, &subst_expr(body, &s1)) {
                        // merge s1 and s2 (s2 is already derived from s1)
                        out.push(merge_subst(&s1, &s2));
                    }
                }
            }
        }
    }
    out
}

// Merge s2 into s1 assuming s2 is downstream of s1
fn merge_subst<'a>(s1: &Subst<'a>, s2: &Subst<'a>) -> Subst<'a> {
    let mut m = s1.clone();
    for (k, v) in s2 { m.insert(k.clone(), v.clone()); }
    m
}


fn subst_term<'a>(t: &Term<'a>, s: &Subst<'a>) -> Term<'a> {
    use Term::*;
    match deref(t.clone(), s) {
        Functor { name, args, span } => {
            Term::Functor { name, args: args.into_iter().map(|a| subst_term(&a, s)).collect(), span }
        }
        ListCons(h, tl, sp) => {
            Term::ListCons(Box::new(subst_term(&h, s)), Box::new(subst_term(&tl, s)), sp)
        }
        other => other,
    }
}

fn subst_expr<'a>(e: &Expr<'a>, s: &Subst<'a>) -> Expr<'a> {
    match e {
        Expr::Term(t, sp) => Expr::Term(subst_term(t, s), sp.clone()),

        Expr::EqualOrBind(l, r, sp) =>
            Expr::EqualOrBind(subst_term(l, s), subst_term(r, s), sp.clone()),

        Expr::NotEqualOrBind(l, r, sp) =>
            Expr::NotEqualOrBind(subst_term(l, s), subst_term(r, s), sp.clone()),

        // NEW
        Expr::NotProvableExpr(inner, sp) =>
            Expr::NotProvableExpr(Box::new(subst_expr(inner, s)), sp.clone()),

        Expr::Cut(sp) => Expr::Cut(sp.clone()),
    }
}

