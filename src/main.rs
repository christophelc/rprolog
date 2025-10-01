use pest::Parser;

fn main() {
    let input = "parent(john, X).";
    let result = rprolog::parser::G::parse(rprolog::parser::Rule::fact, input);
    println!("{:?}", result);
}
