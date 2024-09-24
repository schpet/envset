use chumsky::prelude::*;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum Line {
    Comment(String),
    KeyValue {
        key: String,
        value: String,
        comment: Option<String>,
    },
}

pub fn parser() -> impl Parser<char, Vec<Line>, Error = Simple<char>> + Clone {
    // Parser for comments
    let comment = just('#')
        .ignore_then(take_until(text::newline().or(end())))
        .map(|(chars, _)| chars.into_iter().collect::<String>())
        .map(Line::Comment);

    // Parser for keys
    pub fn key_parser() -> impl Parser<char, String, Error = Simple<char>> + Clone {
        text::ident().padded()
    }

    let key = key_parser();

    // Parser for single-quoted values
    let single_quoted_value = just('\'')
        .ignore_then(filter(|&c| c != '\'').repeated().collect::<String>())
        .then_ignore(just('\''));

    // Parser for escape sequences in double-quoted values
    let escape_sequence = just('\\').then(any());

    // Parser for double-quoted values
    let double_quoted_value = just('"')
        .ignore_then(
            choice((
                escape_sequence.map(|(_, c)| c),
                filter(|&c| c != '"' && c != '\\'),
            ))
            .repeated()
            .collect::<String>(),
        )
        .then_ignore(just('"'));

    // Parser for unquoted values
    let unquoted_value = {
        let escape_sequence = just('\\').then(any()).map(|(_, c)| c);
        let unescaped_char = filter(|&c| c != '#' && c != '\n' && c != '\\');
        choice((escape_sequence, unescaped_char))
            .repeated()
            .collect::<String>()
    };

    let value = choice((single_quoted_value, double_quoted_value, unquoted_value))
        .map(|s| s.trim_end().to_string());

    // Parser for trailing comments
    let trailing_comment = just('#')
        .ignore_then(take_until(text::newline().or(end())))
        .map(|(chars, _)| chars.into_iter().collect::<String>())
        .boxed();

    // Parser for key-value lines
    let key_value_line = key
        .then_ignore(just('='))
        .then(value.padded_by(just(' ').repeated()))
        .then(trailing_comment.or_not())
        .map(|((key, value), comment)| Line::KeyValue {
            key,
            value,
            comment,
        });

    // Parser for a line (either a comment or a key-value pair)
    let line = choice((comment, key_value_line));

    // Parser for the entire file
    line.padded_by(just('\n').repeated()).repeated()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_key_value_pair() {
        let input = "KEY=value\n";
        let result = parser().parse(input).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            Line::KeyValue {
                key,
                value,
                comment,
            } => {
                assert_eq!(key, "KEY");
                assert_eq!(value, "value");
                assert_eq!(comment, &None);
            }
            _ => panic!("Expected KeyValue, got {:?}", result[0]),
        }
    }

    #[test]
    fn test_multiple_key_value_pairs() {
        let input = "KEY1=value1\nKEY2=value2\nKEY3=value3\n";
        let result = parser().parse(input).unwrap();
        assert_eq!(result.len(), 3);

        let expected = vec![("KEY1", "value1"), ("KEY2", "value2"), ("KEY3", "value3")];

        for (i, (expected_key, expected_value)) in expected.iter().enumerate() {
            match &result[i] {
                Line::KeyValue {
                    key,
                    value,
                    comment,
                } => {
                    assert_eq!(key, expected_key);
                    assert_eq!(value, expected_value);
                    assert_eq!(comment, &None);
                }
                _ => panic!("Expected KeyValue, got {:?}", result[i]),
            }
        }
    }

    #[test]
    fn test_whole_line_comment() {
        let input = "# This is a comment\n";
        let result = parser().parse(input).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            Line::Comment(comment) => {
                assert_eq!(comment, " This is a comment");
            }
            _ => panic!("Expected Comment, got {:?}", result[0]),
        }
    }

    #[test]
    fn test_key_value_with_trailing_comment() {
        let input = "KEY=value # This is a trailing comment\n";
        let result = parser().parse(input).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            Line::KeyValue {
                key,
                value,
                comment,
            } => {
                assert_eq!(key, "KEY");
                assert_eq!(value, "value");
                assert_eq!(comment, &Some(" This is a trailing comment".to_string()));
            }
            _ => panic!("Expected KeyValue, got {:?}", result[0]),
        }
    }

    #[test]
    fn test_env_var_with_mixed_comments() {
        let input =
            "# Comment before\nKEY1=value1\n# Comment in between\nKEY2=value2\n# Comment after\n";
        let result = parser().parse(input).unwrap();
        assert_eq!(result.len(), 5);

        match &result[0] {
            Line::Comment(comment) => assert_eq!(comment, " Comment before"),
            _ => panic!("Expected Comment, got {:?}", result[0]),
        }

        match &result[1] {
            Line::KeyValue {
                key,
                value,
                comment,
            } => {
                assert_eq!(key, "KEY1");
                assert_eq!(value, "value1");
                assert_eq!(comment, &None);
            }
            _ => panic!("Expected KeyValue, got {:?}", result[1]),
        }

        match &result[2] {
            Line::Comment(comment) => assert_eq!(comment, " Comment in between"),
            _ => panic!("Expected Comment, got {:?}", result[2]),
        }

        match &result[3] {
            Line::KeyValue {
                key,
                value,
                comment,
            } => {
                assert_eq!(key, "KEY2");
                assert_eq!(value, "value2");
                assert_eq!(comment, &None);
            }
            _ => panic!("Expected KeyValue, got {:?}", result[3]),
        }

        match &result[4] {
            Line::Comment(comment) => assert_eq!(comment, " Comment after"),
            _ => panic!("Expected Comment, got {:?}", result[4]),
        }
    }

    #[test]
    fn test_value_with_trailing_whitespace() {
        let input = "KEY=value with space   \n";
        let result = parser().parse(input).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            Line::KeyValue {
                key,
                value,
                comment,
            } => {
                assert_eq!(key, "KEY");
                assert_eq!(value, "value with space");
                assert_eq!(comment, &None);
            }
            _ => panic!("Expected KeyValue, got {:?}", result[0]),
        }
    }

    #[test]
    fn test_multiline_quoted_value() {
        let input = r#"MULTILINE="
  a multiline comment
  spanning several
  lines
  # not a comment
""#;
        let result = parser().parse(input).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            Line::KeyValue {
                key,
                value,
                comment,
            } => {
                assert_eq!(key, "MULTILINE");
                assert_eq!(
                    value,
                    "\n  a multiline comment\n  spanning several\n  lines\n  # not a comment"
                );
                assert_eq!(comment, &None);
            }
            _ => panic!("Expected KeyValue, got {:?}", result[0]),
        }
    }

    #[test]
    fn test_multiline_json_value() {
        let input = r#"JSON_CONFIG='{
  "key1": "value1",
  "key2": {
    "nested_key": "nested_value"
  },
  "key3": [1, 2, 3]
}'"#;
        let result = parser().parse(input).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            Line::KeyValue {
                key,
                value,
                comment,
            } => {
                assert_eq!(key, "JSON_CONFIG");
                assert_eq!(
                    value,
                    r#"{
  "key1": "value1",
  "key2": {
    "nested_key": "nested_value"
  },
  "key3": [1, 2, 3]
}"#
                );
                assert_eq!(comment, &None);
            }
            _ => panic!("Expected KeyValue, got {:?}", result[0]),
        }
    }
}
