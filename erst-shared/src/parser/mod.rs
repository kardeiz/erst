use pest::Parser;
use crate::err;

#[derive(Parser)]
#[grammar = "parser/erst.pest"]
pub struct ErstParser;