use crate::err;
use pest::Parser;

#[derive(Parser)]
#[grammar = "parser/erst.pest"]
pub struct ErstParser;
