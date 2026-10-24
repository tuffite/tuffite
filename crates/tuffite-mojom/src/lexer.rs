use crate::Error;

pub(crate) fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

#[derive(Clone, Debug)]
pub(crate) struct Token {
    pub(crate) text: String,
    pub(crate) offset: usize,
    pub(crate) length: usize,
}

pub(crate) fn lex(source: &str) -> Result<Vec<Token>, Error> {
    if source.len() > 1024 * 1024 {
        return Err("Mojom source exceeds 1 MiB".into());
    }
    let bytes = source.as_bytes();
    let mut at = 0;
    let mut tokens = Vec::new();
    while at < bytes.len() {
        let start = at;
        match bytes[at] {
            b' ' | b'\r' | b'\t' | b'\n' => {
                at += 1;
            }
            b'/' if bytes.get(at + 1) == Some(&b'/') => {
                while at < bytes.len() && bytes[at] != b'\n' {
                    at += 1;
                }
            }
            b'/' if bytes.get(at + 1) == Some(&b'*') => {
                at += 2;
                while at + 1 < bytes.len() && &bytes[at..at + 2] != b"*/" {
                    at += 1;
                }
                if at + 1 >= bytes.len() {
                    return Err(
                        Error::at("unterminated comment", start, bytes.len() - start)
                            .with_help("Close the comment with */."),
                    );
                }
                at += 2;
            }
            b'"' => {
                at += 1;
                while at < bytes.len() && bytes[at] != b'"' {
                    if bytes[at] == b'\\' {
                        at += 1;
                    }
                    at += 1;
                }
                if at >= bytes.len() {
                    return Err(Error::at("unterminated string", start, bytes.len() - start)
                        .with_help("Close the string with a double quote."));
                }
                at += 1;
                tokens.push(Token {
                    text: source[start..at].into(),
                    offset: start,
                    length: at - start,
                });
            }
            byte if byte.is_ascii_alphanumeric() || byte == b'_' => {
                while at < bytes.len() && (bytes[at].is_ascii_alphanumeric() || bytes[at] == b'_') {
                    at += 1;
                }
                tokens.push(Token {
                    text: source[start..at].into(),
                    offset: start,
                    length: at - start,
                });
            }
            byte if b"{}();,.<>@=?[]".contains(&byte) => {
                at += 1;
                tokens.push(Token {
                    text: source[start..at].into(),
                    offset: start,
                    length: at - start,
                });
            }
            _ => {
                return Err(Error::at(
                    "unsupported character",
                    start,
                    source[start..].chars().next().unwrap().len_utf8(),
                ));
            }
        }
    }
    tokens.push(Token {
        text: "<eof>".into(),
        offset: source.len(),
        length: 0,
    });
    Ok(tokens)
}
