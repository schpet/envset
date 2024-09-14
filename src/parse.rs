use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Node {
    KeyValue {
        key: String,
        value: String,
        trailing_comment: Option<String>,
    },
    Comment(String),
    EmptyLine,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Ast {
    nodes: Vec<Node>,
}

impl Ast {
    pub fn new() -> Self {
        Ast { nodes: Vec::new() }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }
}

pub fn parse(input: &str) -> Ast {
    let mut ast = Ast::new();
    for line in input.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            ast.add_node(Node::EmptyLine);
        } else if trimmed.starts_with('#') {
            ast.add_node(Node::Comment(line.to_string()));
        } else {
            let (key, value, comment) = parse_key_value(line);
            ast.add_node(Node::KeyValue {
                key,
                value,
                trailing_comment: comment,
            });
        }
    }
    ast
}

fn parse_key_value(line: &str) -> (String, String, Option<String>) {
    let mut parts = line.splitn(2, '=');
    let key = parts.next().unwrap().trim().to_string();
    let value_and_comment = parts.next().unwrap_or("").trim_start();

    let (value, comment) = split_value_and_comment(value_and_comment);

    (key, value, comment)
}

fn split_value_and_comment(s: &str) -> (String, Option<String>) {
    let mut in_quotes = false;
    let mut escape = false;
    let mut value = String::new();
    let mut comment = String::new();

    for (_i, c) in s.char_indices() {
        match c {
            '#' if !in_quotes && !escape => {
                comment = s[_i..].to_string();
                break;
            }
            '"' if !escape => in_quotes = !in_quotes,
            '\\' if !escape => escape = true,
            _ => {
                if escape {
                    escape = false;
                }
                value.push(c);
            }
        }
    }

    let value = value.trim().to_string();
    let comment = if comment.is_empty() {
        None
    } else {
        Some(comment.trim().to_string())
    };

    (value, comment)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let input = r#"
# This is a comment
KEY1=value1
KEY2="value2" # This is a trailing comment
KEY3=value3#not a comment
KEY4="value4#still not a comment"
# Another comment
KEY5=value5

KEY6="value6"
"#;
        let ast = parse(input);
        assert_eq!(
            ast.nodes,
            vec![
                Node::EmptyLine,
                Node::Comment("# This is a comment".to_string()),
                Node::KeyValue {
                    key: "KEY1".to_string(),
                    value: "value1".to_string(),
                    trailing_comment: None,
                },
                Node::KeyValue {
                    key: "KEY2".to_string(),
                    value: "value2".to_string(),
                    trailing_comment: Some("# This is a trailing comment".to_string()),
                },
                Node::KeyValue {
                    key: "KEY3".to_string(),
                    value: "value3#not a comment".to_string(),
                    trailing_comment: None,
                },
                Node::KeyValue {
                    key: "KEY4".to_string(),
                    value: "value4#still not a comment".to_string(),
                    trailing_comment: None,
                },
                Node::Comment("# Another comment".to_string()),
                Node::KeyValue {
                    key: "KEY5".to_string(),
                    value: "value5".to_string(),
                    trailing_comment: None,
                },
                Node::EmptyLine,
                Node::KeyValue {
                    key: "KEY6".to_string(),
                    value: "value6".to_string(),
                    trailing_comment: None,
                },
            ]
        );
    }
}
