use chumsky::prelude::*;

#[derive(Debug)]
pub enum Line {
    Comment(String),
    KeyValue {
        key: String,
        value: String,
        comment: Option<String>,
    },
}

fn parser() -> impl Parser<char, Vec<Line>, Error = Simple<char>> {
    // Parser for comments
    let comment = just('#')
        .ignore_then(take_until(text::newline().or(end())))
        .collect::<String>()
        .map(Line::Comment);

    // Parser for keys
    let key = text::ident().padded();

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

    let value = choice((single_quoted_value, double_quoted_value, unquoted_value));

    // Parser for trailing comments
    let trailing_comment = just('#')
        .ignore_then(take_until(text::newline().or(end())))
        .collect::<String>()
        .boxed();

    // Parser for key-value lines
    let key_value_line = key
        .then_ignore(just('=').padded())
        .then(value.then(trailing_comment.or_not()))
        .map(|(key, (value, comment))| Line::KeyValue {
            key,
            value,
            comment,
        });

    // Parser for a line (either a comment or a key-value pair)
    let line = choice((comment, key_value_line)).then_ignore(text::newline().or(end()));

    // Parser for the entire file
    line.repeated()
}
