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
    pub nodes: Vec<Node>,
}

impl Ast {
    pub fn new() -> Self {
        Ast { nodes: Vec::new() }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Node> {
        self.nodes.iter()
    }

    pub fn first(&self) -> Option<&Node> {
        self.nodes.first()
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
    let mut key = String::new();
    let mut value = String::new();
    let mut comment = None;
    let mut chars = line.chars().peekable();
    let mut in_key = true;
    let mut in_value = false;
    let mut in_strong_quote = false;
    let mut in_weak_quote = false;
    let mut escaped = false;
    while let Some(c) = chars.next() {
        if in_key {
            if c.is_ascii_alphanumeric() || c == '_' || c == '.' {
                key.push(c);
            } else if c == '=' {
                in_key = false;
                in_value = true;
            } else if c.is_whitespace() && key == "export" {
                key.clear();
            } else if !c.is_whitespace() {
                // Invalid key character
                return (String::new(), String::new(), None);
            }
        } else if in_value {
            if escaped {
                match c {
                    '\\' | '\'' | '"' | '$' | ' ' => value.push(c),
                    'n' => value.push('\n'),
                    _ => {
                        // Invalid escape sequence
                        return (String::new(), String::new(), None);
                    }
                }
                escaped = false;
            } else if in_strong_quote {
                if c == '\'' {
                    in_strong_quote = false;
                } else {
                    value.push(c);
                }
            } else if in_weak_quote {
                if c == '"' {
                    in_weak_quote = false;
                } else if c == '\\' {
                    escaped = true;
                } else {
                    value.push(c);
                }
            } else {
                match c {
                    '\'' => in_strong_quote = true,
                    '"' => in_weak_quote = true,
                    '\\' => escaped = true,
                    '#' => {
                        comment = Some(format!("#{}", chars.collect::<String>()));
                        break;
                    }
                    ' ' | '\t' if value.is_empty() => continue, // Skip leading whitespace
                    ' ' | '\t' => {
                        // Check if there's a comment after whitespace
                        if let Some('#') = chars.peek() {
                            chars.next(); // consume '#'
                            comment = Some(format!("#{}", chars.collect::<String>()));
                            break;
                        }
                        value.push(c);
                    }
                    _ => value.push(c),
                }
            }
        }
    }

    if in_strong_quote || in_weak_quote || escaped {
        // Unclosed quotes or trailing backslash
        return (String::new(), String::new(), None);
    }

    (key.trim().to_string(), value.trim().to_string(), comment)
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
                    value: "value3".to_string(),
                    trailing_comment: Some("#not a comment".to_string()),
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

    #[test]
    fn test_parse_line_env() {
        let input = r#"
KEY=1
KEY2="2"
KEY3='3'
KEY4='fo ur'
KEY5="fi ve"
KEY6=s\ ix
KEY7=
KEY8=
KEY9=   # foo
KEY10  ="whitespace before ="
KEY11=    "whitespace after ="
export="export as key"
export   SHELL_LOVER=1
"#;
        let ast = parse(input);
        let expected = vec![
            Node::EmptyLine,
            Node::KeyValue {
                key: "KEY".to_string(),
                value: "1".to_string(),
                trailing_comment: None,
            },
            Node::KeyValue {
                key: "KEY2".to_string(),
                value: "2".to_string(),
                trailing_comment: None,
            },
            Node::KeyValue {
                key: "KEY3".to_string(),
                value: "3".to_string(),
                trailing_comment: None,
            },
            Node::KeyValue {
                key: "KEY4".to_string(),
                value: "fo ur".to_string(),
                trailing_comment: None,
            },
            Node::KeyValue {
                key: "KEY5".to_string(),
                value: "fi ve".to_string(),
                trailing_comment: None,
            },
            Node::KeyValue {
                key: "KEY6".to_string(),
                value: "s ix".to_string(),
                trailing_comment: None,
            },
            Node::KeyValue {
                key: "KEY7".to_string(),
                value: "".to_string(),
                trailing_comment: None,
            },
            Node::KeyValue {
                key: "KEY8".to_string(),
                value: "".to_string(),
                trailing_comment: None,
            },
            Node::KeyValue {
                key: "KEY9".to_string(),
                value: "".to_string(),
                trailing_comment: Some("# foo".to_string()),
            },
            Node::KeyValue {
                key: "KEY10".to_string(),
                value: "whitespace before =".to_string(),
                trailing_comment: None,
            },
            Node::KeyValue {
                key: "KEY11".to_string(),
                value: "whitespace after =".to_string(),
                trailing_comment: None,
            },
            Node::KeyValue {
                key: "export".to_string(),
                value: "export as key".to_string(),
                trailing_comment: None,
            },
            Node::KeyValue {
                key: "SHELL_LOVER".to_string(),
                value: "1".to_string(),
                trailing_comment: None,
            },
        ];
        assert_eq!(ast.nodes, expected);
    }

    #[test]
    fn test_parse_value_escapes() {
        let input = r#"
KEY=my\ cool\ value
KEY2=\$sweet
KEY3="awesome stuff \"mang\""
KEY4='sweet $\fgs'\''fds'
KEY5="'\"yay\\"\ "stuff"
KEY6="lol" #well you see when I say lol wh
KEY7="line 1\nline 2"
"#;
        let ast = parse(input);
        let expected = vec![
            Node::EmptyLine,
            Node::KeyValue {
                key: "KEY".to_string(),
                value: "my cool value".to_string(),
                trailing_comment: None,
            },
            Node::KeyValue {
                key: "KEY2".to_string(),
                value: "$sweet".to_string(),
                trailing_comment: None,
            },
            Node::KeyValue {
                key: "KEY3".to_string(),
                value: r#"awesome stuff "mang""#.to_string(),
                trailing_comment: None,
            },
            Node::KeyValue {
                key: "KEY4".to_string(),
                value: "sweet $\\fgs'fds".to_string(),
                trailing_comment: None,
            },
            Node::KeyValue {
                key: "KEY5".to_string(),
                value: r#"'"yay\ stuff"#.to_string(),
                trailing_comment: None,
            },
            Node::KeyValue {
                key: "KEY6".to_string(),
                value: "lol".to_string(),
                trailing_comment: Some("#well you see when I say lol wh".to_string()),
            },
            Node::KeyValue {
                key: "KEY7".to_string(),
                value: "line 1\nline 2".to_string(),
                trailing_comment: None,
            },
        ];
        assert_eq!(ast.nodes, expected);
    }
}
