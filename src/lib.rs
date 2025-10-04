pub mod ast;
pub mod parser;
pub mod tree;
pub mod engine;

use pest::Parser;
use crate::parser::{Rule, G};

pub fn parse_program<'a>(src: &'a str) -> Result<ast::Program<'a>, pest::error::Error<parser::Rule>> {    
    let mut pairs = G::parse(Rule::program, src)?;
    let top = pairs.next().unwrap();
    tree::build_program(top)
}
