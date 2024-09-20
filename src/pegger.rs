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

        rule eol() = "\n" / "\r\n"

        rule comment() -> &'input str
            = "#" s:$((!eol() [_])*) { s }

        rule key() -> &'input str
            = s:$([a-zA-Z_] [a-zA-Z0-9_]*) { s }

        rule value() -> &'input str
            = s:$((!comment() !eol() [_])*) { s.trim() }

        rule pair() -> EnvEntry<'input>
            = k:key() _ "=" _ v:value() _ c:comment()? eol() { EnvEntry::KeyValue(k, v, c) }

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
