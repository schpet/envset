use peg::error::ParseError;

#[derive(Debug)]
pub enum EnvLine {
    KeyValue {
        key: String,
        value: String,
        comment: Option<String>, // Trailing comment
    },
    Comment(String), // Whole-line comment
}

peg::parser!{
    // TODO
}
