#[derive(erst::Template)]
#[template(path = "test.erst", type = "html")]
pub struct Thing { pub collection: Vec<String> }
