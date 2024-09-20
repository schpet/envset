use chumsky::prelude::*;

#[derive(Debug, Clone, PartialEq)]
enum EnvEntry {
    KeyValue {
        key: String,
        value: String,
        comment: Option<String>,
    },
    Comment(String),
}

fn env_parser() -> impl Parser<char, Vec<EnvEntry>, Error = Simple<char>> {
    // Parse a key (alphanumeric or underscore)
    let key = filter(|c: &char| c.is_alphanumeric() || *c == '_')
        .repeated()
        .at_least(1)
        .collect::<String>()
        .labelled("key");

    // Parse escaped character
    let escaped_char = just('\\').then(any()).map(|(_, c)| c);

    // Parse single-quoted value
    let single_quoted = just('\'')
        .ignore_then(escaped_char.or(filter(|c| *c != '\'')).repeated())
        .then_ignore(just('\''))
        .collect::<String>();

    // Parse double-quoted value
    let double_quoted = just('"')
        .ignore_then(escaped_char.or(filter(|c| *c != '"')).repeated())
        .then_ignore(just('"'))
        .collect::<String>();

    // Parse unquoted value
    let unquoted = filter(|c: &char| !c.is_whitespace() && *c != '#')
        .repeated()
        .collect::<String>();

    // Parse value (single-quoted, double-quoted, or unquoted)
    let value = single_quoted
        .or(double_quoted)
        .or(unquoted)
        .map(|s: String| unescape(&s))
        .labelled("value");

    // Parse comment
    let comment = just('#')
        .ignore_then(take_until(end().or(just('\n'))))
        .map(|chars: Vec<char>| chars.into_iter().collect::<String>())
        .or(end().to(String::new()))
        .padded();

    // Parse a key-value pair with optional comment
    let pair = key
        .then_ignore(just('='))
        .then(value)
        .then(comment.or_not())
        .map(|((k, v), c)| EnvEntry::KeyValue {
            key: k,
            value: v,
            comment: c,
        });

    // Parse a line (key-value pair, comment, or empty line)
    let line = pair
        .or(comment.map(EnvEntry::Comment))
        .or(just('\n').to(EnvEntry::Comment(String::new())))
        .or(end().to(EnvEntry::Comment(String::new())));

    // Parse the entire file
    line.padded().repeated().then_ignore(end())
}

fn unescape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next_ch) = chars.next() {
                result.push(match next_ch {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    _ => next_ch,
                });
            }
        } else {
            result.push(ch);
        }
    }
    result
}

fn parse_env(input: &str) -> Result<Vec<EnvEntry>, Vec<Simple<char>>> {
    env_parser().parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_key() {
        let input = "KEY=value\n";
        let result = parse_env(input).unwrap();
        assert_eq!(
            result,
            vec![EnvEntry::KeyValue {
                key: "KEY".to_string(),
                value: "value".to_string(),
                comment: None,
            }]
        );
    }

    #[test]
    fn test_parse_multiple_keys() {
        let input = "KEY1=value1\nKEY2=value2\nKEY3=value3\n";
        let result = parse_env(input).unwrap();
        assert_eq!(
            result,
            vec![
                EnvEntry::KeyValue {
                    key: "KEY1".to_string(),
                    value: "value1".to_string(),
                    comment: None,
                },
                EnvEntry::KeyValue {
                    key: "KEY2".to_string(),
                    value: "value2".to_string(),
                    comment: None,
                },
                EnvEntry::KeyValue {
                    key: "KEY3".to_string(),
                    value: "value3".to_string(),
                    comment: None,
                }
            ]
        );
    }

    #[test]
    fn test_parse_quoted_value() {
        let input =
            r#"KEY1="quoted value"\nKEY2='single quoted'\nKEY3="value with \"escaped\" quotes""#;
        let result = parse_env(input).unwrap();
        assert_eq!(
            result,
            vec![
                EnvEntry::KeyValue {
                    key: "KEY1".to_string(),
                    value: "quoted value".to_string(),
                    comment: None,
                },
                EnvEntry::KeyValue {
                    key: "KEY2".to_string(),
                    value: "single quoted".to_string(),
                    comment: None,
                },
                EnvEntry::KeyValue {
                    key: "KEY3".to_string(),
                    value: r#"value with "escaped" quotes"#.to_string(),
                    comment: None,
                }
            ]
        );
    }
}
