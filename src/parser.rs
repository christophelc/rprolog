use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct G;

#[cfg(test)]    
mod tests {
    use crate::tree::parse_program;

    use super::*;

    #[test]
    fn test_grammar() {
        let ok = [
            "0", "-0", "12", "-3.14", ".5", "1e10", "6.02E23", "1e-9", "10E+3f", "2.0d"
        ];
        let bad = ["-", ".", "e10", "3e", "1..2", ""];

        for s in ok {
            let parsed = G::parse(Rule::number, s);
            assert!(
                parsed.is_ok() && parsed.unwrap().as_str() == s,
                "Expected '{s}' to parse as number, but got error or partial parse"
            );
        }

        for s in bad {
            let parsed = G::parse(Rule::number, s);
            // Accept only if the parse fails or does not consume the whole input
            if let Ok(mut pairs) = parsed {
                let pair = pairs.next().unwrap();
                assert!(
                    pair.as_span().as_str() != s,
                    "Expected '{s}' to fail parsing as number, but got: {pair:?}"
                );
            }
        }
    }

    #[test]
    fn test_number() {
        let ok = [
            "0", "-0", "12", "-3.14", ".5", "1e10", "6.02E23", "1e-9", "10E+3f", "2.0d"
        ];
        let bad = ["-", ".", "e10", "3e", "1..2", ""];

        for s in ok {
            let parsed = G::parse(Rule::number, s);
            assert!(
                parsed.is_ok() && parsed.unwrap().as_str() == s,
                "Expected '{s}' to parse as number, but got error or partial parse"
            );
        }

        for s in bad {
            let parsed = G::parse(Rule::number, s);
            if let Ok(mut pairs) = parsed {
                let pair = pairs.next().unwrap();
                assert!(
                    pair.as_span().as_str() != s,
                    "Expected '{s}' to fail parsing as number, but got: {pair:?}"
                );
            }
        }
    }

    #[test]
    fn test_atom() {
        let ok = ["foo", "'bar'", "\"baz\"", "abc123", "'with space'"];
        for s in ok {
            let parsed = G::parse(Rule::atom, s);
            assert!(
                parsed.is_ok() && parsed.unwrap().as_str() == s,
                "Expected '{s}' to parse as atom"
            );
        }
    }

    #[test]
    fn test_variable() {
        let ok = ["X", "Var1", "_", "_foo"];
        let bad = ["x", "1X", "foo_bar"];
        for s in ok {
            let parsed = G::parse(Rule::variable, s);
            assert!(
                parsed.is_ok() && parsed.unwrap().as_str() == s,
                "Expected '{s}' to parse as variable"
            );
        }
        for s in bad {
            let parsed = G::parse(Rule::variable, s);
            if let Ok(mut pairs) = parsed {
                let pair = pairs.next().unwrap();
                assert!(
                    pair.as_span().as_str() != s,
                    "Expected '{s}' to fail parsing as variable, but got: {pair:?}"
                );
            }
        }
    }

    #[test]
    fn test_functor() {
        let ok = ["foo(X)", "bar(1, Y)", "'baz'()", "f(a,b,c)"];
        for s in ok {
            let parsed = G::parse(Rule::functor, s);
            assert!(
                parsed.is_ok() && parsed.unwrap().as_str() == s,
                "Expected '{s}' to parse as functor"
            );
        }
    }

    #[test]
    fn test_list() {
        let ok = ["[]", "[a]", "[a,b]", "[X|Y]", "[a,b|T]"];
        for s in ok {
            let parsed = G::parse(Rule::list, s);
            assert!(
                parsed.is_ok() && parsed.unwrap().as_str() == s,
                "Expected '{s}' to parse as list"
            );
        }
    }

    #[test]
    fn test_fact() {
        let ok = ["foo(bar).", "parent(john, X)."];
        for s in ok {
            let parsed = G::parse(Rule::fact, s);
            assert!(
                parsed.is_ok() && parsed.unwrap().as_str() == s,
                "Expected '{s}' to parse as fact"
            );
        }
    }

    #[test]
    fn test_functor_ws() {
        let s = "parent(mary, X)";
        let parsed = G::parse(Rule::functor, s);
        assert!(parsed.is_ok(), "Failed to parse functor: {:?}", parsed);
    }
    
    #[test]
    fn test_rule_with_not() {
        let s = "parent(mary, X) :- \\+ X = john.";
        let parsed = G::parse(Rule::rule, s);
        println!("{:?}", parsed);
        assert!(parsed.is_ok(), "Failed to parse: {:?}", parsed);
    }
    #[test]
    fn test_program() {
        let src = r#"
  parent(john, X).
  parent(mary, X) :- \+ X = john.
  ?- parent(john, Y).
  "#;
        let prog = parse_program(src).unwrap();
        println!("{:#?}", prog);
    }
}