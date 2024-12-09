use yggdrasil_grammar::parse;
#[test]
fn test() {
    let thing = "∀x ∀y (x < y ↔ ∃z ((x + s(z)) = y))";

    let expr = parse(thing).unwrap();

    println!("{:#?}", expr);
}
