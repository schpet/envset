#[derive(Debug, PartialEq)]
pub enum EnvLine {
    KeyValue {
        key: String,
        value: String,
        comment: Option<String>, // Trailing comment
    },
    Comment(String), // Whole-line comment
    EmptyLine,
}

peg::parser! {
    pub grammar env_parser() for str {
        pub rule file() -> Vec<EnvLine>
            = lines:((line() eol())* line()?) {
                lines.into_iter().flatten().collect()
            }

        rule eol() = ['\n' | '\r' | '\r\n']

        pub rule line() -> EnvLine
            = comment()
            / key_value()
            / empty_line()
            / s:$([^'\n']+) { EnvLine::KeyValue { key: "".to_string(), value: s.trim().to_string(), comment: None } }

        rule comment() -> EnvLine
            = "#" s:$([^'\n']*) { EnvLine::Comment(s.to_string()) }

        rule key_value() -> EnvLine
            = k:key() "=" v:value() c:trailing_comment()? {
                EnvLine::KeyValue {
                    key: k.to_string(),
                    value: v.trim_end().to_string(),
                    comment: c.map(|s| s.to_string()),
                }
            }

        rule key() -> &'input str
            = $(['a'..='z' | 'A'..='Z' | '0'..='9' | '_']+)

        rule value() -> &'input str
            = quoted_value()
            / unquoted_value()

        rule quoted_value() -> &'input str
            = "\"" v:$((!['"'][_] / "\\\\" / "\\\"" )*) "\"" { v }
            / "'" v:$((!['\''][_] / "\\\\" / "\\'" )*) "'" { v }

        rule unquoted_value() -> &'input str
            = $([^'#' | '\n']*)

        rule trailing_comment() -> &'input str
            = [' ' | '\t']* "#" s:$([^'\n']*) { s }

        rule empty_line() -> EnvLine
            = [' ' | '\t']* "\n"? { EnvLine::EmptyLine }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_value() {
        assert_eq!(
            env_parser::line("FOO=bar"),
            Ok(EnvLine::KeyValue {
                key: "FOO".to_string(),
                value: "bar".to_string(),
                comment: None,
            })
        );
    }

    #[test]
    fn test_quoted_values() {
        assert_eq!(
            env_parser::line(r#"A='foo'"#),
            Ok(EnvLine::KeyValue {
                key: "A".to_string(),
                value: "foo".to_string(),
                comment: None,
            })
        );
        assert_eq!(
            env_parser::line(r#"B="foo""#),
            Ok(EnvLine::KeyValue {
                key: "B".to_string(),
                value: "foo".to_string(),
                comment: None,
            })
        );
        assert_eq!(
            env_parser::line(r#"C='foo"bar'"#),
            Ok(EnvLine::KeyValue {
                key: "C".to_string(),
                value: r#"foo"bar"#.to_string(),
                comment: None,
            })
        );
        assert_eq!(
            env_parser::line(r#"D="foo\"bar""#),
            Ok(EnvLine::KeyValue {
                key: "D".to_string(),
                value: r#"foo"bar"#.to_string(),
                comment: None,
            })
        );
        assert_eq!(
            env_parser::line(r#"E='foo\'bar'"#),
            Ok(EnvLine::KeyValue {
                key: "E".to_string(),
                value: r#"foo'bar"#.to_string(),
                comment: None,
            })
        );
    }

    #[test]
    fn test_comment() {
        assert_eq!(
            env_parser::line("# This is a comment"),
            Ok(EnvLine::Comment(" This is a comment".to_string()))
        );
    }

    #[test]
    fn test_key_value_with_trailing_comment() {
        assert_eq!(
            env_parser::line("FOO=bar # This is a comment"),
            Ok(EnvLine::KeyValue {
                key: "FOO".to_string(),
                value: "bar".to_string(),
                comment: Some(" This is a comment".to_string()),
            })
        );
    }

    #[test]
    fn test_empty_line() {
        assert_eq!(env_parser::line(""), Ok(EnvLine::EmptyLine));
        assert_eq!(env_parser::line("  \t  "), Ok(EnvLine::EmptyLine));
    }

    #[test]
    fn test_multiple_lines() {
        let input = r#"
FOO=bar
# This is a comment
KEY=value with spaces
EMPTY=
QUOTED='single quoted'
This line has no equals sign
"#;
        let result = env_parser::file(input);
        assert!(result.is_ok(), "Failed to parse multiple lines: {:?}", result.err());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 7, "Expected 7 lines, got {}", lines.len());
        
        assert_eq!(lines[0], EnvLine::EmptyLine, "First line should be empty");
        assert_eq!(
            lines[1],
            EnvLine::KeyValue {
                key: "FOO".to_string(),
                value: "bar".to_string(),
                comment: None,
            },
            "Second line should be FOO=bar"
        );
        assert_eq!(lines[2], EnvLine::Comment(" This is a comment".to_string()), "Third line should be a comment");
        assert_eq!(
            lines[3],
            EnvLine::KeyValue {
                key: "KEY".to_string(),
                value: "value with spaces".to_string(),
                comment: None,
            },
            "Fourth line should be KEY=value with spaces"
        );
        assert_eq!(
            lines[4],
            EnvLine::KeyValue {
                key: "EMPTY".to_string(),
                value: "".to_string(),
                comment: None,
            },
            "Fifth line should be EMPTY="
        );
        assert_eq!(
            lines[5],
            EnvLine::KeyValue {
                key: "QUOTED".to_string(),
                value: "single quoted".to_string(),
                comment: None,
            },
            "Sixth line should be QUOTED='single quoted'"
        );
        assert_eq!(
            lines[6],
            EnvLine::KeyValue {
                key: "".to_string(),
                value: "This line has no equals sign".to_string(),
                comment: None,
            },
            "Seventh line should be 'This line has no equals sign'"
        );
    }
}
