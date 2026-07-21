use crossterm::style::Attribute;

#[derive(Clone, Copy)]
pub struct Span<'a> {
    pub text: &'a str,
    pub attribute: Attribute,
}

/// A small line-oriented Markdown parser for terminal-friendly formatting.
pub fn spans(line: &str) -> Vec<Span<'_>> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
        return vec![span(line, Attribute::Dim)];
    }
    if trimmed.starts_with('#') {
        let hashes = trimmed.chars().take_while(|c| *c == '#').count();
        if hashes <= 6 && trimmed.chars().nth(hashes) == Some(' ') {
            return vec![span(line, Attribute::Bold)];
        }
    }
    if trimmed.starts_with("> ") {
        return vec![span(line, Attribute::Italic)];
    }
    let mut out = Vec::new();
    let mut rest = line;
    while !rest.is_empty() {
        let marker = ["**", "__", "`", "*", "_"]
            .into_iter()
            .filter_map(|m| rest.find(m).map(|i| (i, m)))
            .min_by_key(|x| x.0);
        let Some((start, mark)) = marker else {
            out.push(span(rest, Attribute::Reset));
            break;
        };
        if start > 0 {
            out.push(span(&rest[..start], Attribute::Reset));
        }
        let after = &rest[start + mark.len()..];
        if let Some(end) = after.find(mark) {
            let finish = start + mark.len() + end + mark.len();
            let attr = if mark == "`" {
                Attribute::Underlined
            } else if mark.len() == 2 {
                Attribute::Bold
            } else {
                Attribute::Italic
            };
            out.push(span(&rest[start..finish], attr));
            rest = &rest[finish..];
        } else {
            out.push(span(rest, Attribute::Reset));
            break;
        }
    }
    out
}

fn span(text: &str, attribute: Attribute) -> Span<'_> {
    Span { text, attribute }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_inline_emphasis() {
        let parsed = spans("one **two** three");
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[1].attribute, Attribute::Bold);
    }
}
