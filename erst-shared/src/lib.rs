#[macro_use]
extern crate pest_derive;

pub mod parser {
    #[derive(Parser)]
    #[grammar = "erst.pest"]
    pub struct ErstParser;
}

pub mod exp {
    pub use pest::Parser;
}
