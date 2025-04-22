use chumsky::Parser;

#[test]
fn test() {
    // let thing = "∀x ∀y (x < y ↔ ∃z ((x + s(z)) = y))";

    let input = "(A | B | C) &( @x (P(y)))";

    let (out, err) = yggdrasil_grammar::PARSER.with(|parser| {
        let parser = parser.get();
        parser.parse(input).into_output_errors()
    });

    println!("expr:\n{:#?}\n\nerr:\n{:?}", out, err);

    assert!(err.is_empty());
}
