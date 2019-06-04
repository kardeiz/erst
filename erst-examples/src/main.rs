use erst::Template;

fn main() {

	// erst::dynamic::compile("test2.rs").unwrap();

	println!("{}", (erst_examples::Thing { collection: vec!["<>".into(), "World".into()]}));

}