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
    grammar env_parser() for str {

        use std::str::FromStr;

        pub rule parse() -> Vec<EnvLine>
            = _ lines:(env_line() ** ("\r\n" / "\n")) _ {
                lines
            }

        rule env_line() -> EnvLine
            = c:comment() {
                EnvLine::Comment(c)
            }
            / kv:key_value() {
                kv
            }

        rule key_value() -> EnvLine
            = key:key() _ "=" _ value:value() _ comment:trailing_comment()? {
                EnvLine::KeyValue {
                    key,
                    value,
                    comment,
                }
            }

        rule key() -> String
            = s:$([a-zA-Z0-9_]+) { s.to_string() }

        rule value() -> String
            = quoted_value() / unquoted_value()

        rule quoted_value() -> String
            = q:("\"") v:double_quoted_content() "\"" { v }

        rule double_quoted_content() -> String
            = s:(double_quoted_char()*) { s.concat() }

        rule double_quoted_char() -> String
            = "\\" c:['\\' | '"' | 'n' | 'r' | 't'] {
                match c {
                    '\\' => "\\".to_string(),
                    '"' => "\"".to_string(),
                    'n' => "\n".to_string(),
                    'r' => "\r".to_string(),
                    't' => "\t".to_string(),
                    _ => c.to_string(),
                }
            }
            / [^"\\]

        rule unquoted_value() -> String
            = s:$([^#\n\r]*) { s.trim().to_string() }

        rule comment() -> String
            = full_comment:whole_comment() { full_comment }

        rule whole_comment() -> String
            = _ "#" c:$( [^\n\r]* ) { c.to_string().trim().to_string() }

        rule trailing_comment() -> String
            = _ "#" c:$( [^\n\r]* ) { c.to_string().trim().to_string() }

        // Whitespace and line breaks
        rule _ = [ \t]*

    }
}
