use peg;

#[derive(Debug)]
pub enum EnvEntry<'a> {
    KeyValue(&'a str, &'a str, Option<&'a str>), // (key, value, trailing_comment)
    Comment(&'a str),
    EmptyLine,
}

peg::parser! {
    pub grammar env_parser() for str {
        rule _() = [' ' | '\t']*

        rule eol() = "\n" / "\r\n" / ![_]

        rule comment() -> &'input str
            = "#" s:$((!eol() [_])*) { s }

        rule key() -> &'input str
            = s:$(['a'..='z' | 'A'..='Z' | '_']['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.']*) { s }

        rule unquoted_value() -> &'input str
            = s:$((!comment() !eol() [^ '\t' | ' ' | '"' | '\'']+)) { s.trim() }

        rule quoted_value() -> String
            = "\"" v:$((!['"' | '\\'] [_] / "\\\\" / "\\\"" )*) "\"" { v.replace("\\\"", "\"").replace("\\\\", "\\") }
            / "'" v:$((!['\'' | '\\'] [_] / "\\\\" / "\\'" )*) "'" { v.replace("\\'", "'").replace("\\\\", "\\") }

        rule value() -> String
            = v:(quoted_value() / unquoted_value()) { v.to_string() }

        rule pair() -> EnvEntry<'input>
            = k:key() _ "=" _ v:value() _ c:comment()? eol() { EnvEntry::KeyValue(k, &v, c) }

        rule comment_line() -> EnvEntry<'input>
            = _ c:comment() eol() { EnvEntry::Comment(c) }

        rule empty_line() -> EnvEntry<'input>
            = _ eol() { EnvEntry::EmptyLine }

        rule line() -> EnvEntry<'input>
            = pair() / comment_line() / empty_line()

        pub rule file() -> Vec<EnvEntry<'input>>
            = lines:line()* { lines }
    }
}
