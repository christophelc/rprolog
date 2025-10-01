use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct G;

#[cfg(test)]    
mod tests {
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
}
