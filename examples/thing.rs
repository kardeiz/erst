use erst::Template;
#[derive(Template)]
#[template(path = "test.erst", type = "html")]
pub struct Thing { pub collection: Vec<String> }

fn main() {
	println!("{}", (Thing { collection: vec!["<>".into(), "World".into()]}));
}