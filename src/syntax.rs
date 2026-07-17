use std::path::Path;

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Plain,
    Keyword,
    String,
    Comment,
    Number,
    Type,
    Function,
}

#[derive(Clone)]
pub struct Language {
    pub name: &'static str,
    keywords: &'static [&'static str],
    line_comment: &'static str,
}

const PYTHON: &[&str] = &[
    "and", "as", "assert", "async", "await", "break", "case", "class", "continue", "def", "del",
    "elif", "else", "except", "False", "finally", "for", "from", "global", "if", "import", "in",
    "is", "lambda", "match", "None", "nonlocal", "not", "or", "pass", "raise", "return", "True",
    "try", "while", "with", "yield",
];
const RUST: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
];
const JAVA: &[&str] = &[
    "abstract",
    "assert",
    "boolean",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extends",
    "false",
    "final",
    "finally",
    "float",
    "for",
    "if",
    "implements",
    "import",
    "instanceof",
    "int",
    "interface",
    "long",
    "native",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "short",
    "static",
    "strictfp",
    "super",
    "switch",
    "synchronized",
    "this",
    "throw",
    "throws",
    "transient",
    "true",
    "try",
    "void",
    "volatile",
    "while",
];
const LUA: &[&str] = &[
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto", "if", "in",
    "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
];
const SCALA: &[&str] = &[
    "abstract",
    "case",
    "catch",
    "class",
    "def",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "final",
    "finally",
    "for",
    "given",
    "if",
    "implicit",
    "import",
    "lazy",
    "match",
    "new",
    "null",
    "object",
    "opaque",
    "override",
    "package",
    "private",
    "protected",
    "return",
    "sealed",
    "super",
    "then",
    "this",
    "throw",
    "trait",
    "transparent",
    "true",
    "try",
    "type",
    "val",
    "var",
    "while",
    "with",
    "yield",
];
const C: &[&str] = &[
    "alignas",
    "alignof",
    "auto",
    "bool",
    "break",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "constexpr",
    "continue",
    "default",
    "delete",
    "do",
    "double",
    "else",
    "enum",
    "explicit",
    "export",
    "extern",
    "false",
    "float",
    "for",
    "friend",
    "goto",
    "if",
    "inline",
    "int",
    "long",
    "namespace",
    "new",
    "nullptr",
    "operator",
    "private",
    "protected",
    "public",
    "register",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "struct",
    "switch",
    "template",
    "this",
    "throw",
    "true",
    "try",
    "typedef",
    "typename",
    "union",
    "unsigned",
    "using",
    "virtual",
    "void",
    "volatile",
    "while",
];

pub fn detect(path: &Path, override_name: Option<&str>) -> Option<Language> {
    let ext = override_name
        .map(str::to_owned)
        .or_else(|| path.extension()?.to_str().map(str::to_owned))?
        .to_ascii_lowercase();
    match ext.as_str() {
        "py" | "python" => Some(Language {
            name: "Python",
            keywords: PYTHON,
            line_comment: "#",
        }),
        "rs" | "rust" => Some(Language {
            name: "Rust",
            keywords: RUST,
            line_comment: "//",
        }),
        "java" => Some(Language {
            name: "Java",
            keywords: JAVA,
            line_comment: "//",
        }),
        "lua" => Some(Language {
            name: "Lua",
            keywords: LUA,
            line_comment: "--",
        }),
        "scala" | "sc" => Some(Language {
            name: "Scala",
            keywords: SCALA,
            line_comment: "//",
        }),
        "c" | "h" | "cc" | "cpp" | "cxx" | "hpp" | "hxx" | "c++" => Some(Language {
            name: "C/C++",
            keywords: C,
            line_comment: "//",
        }),
        _ => None,
    }
}

pub fn highlight(line: &str, lang: Option<&Language>) -> Vec<(String, Kind)> {
    let Some(lang) = lang else {
        return vec![(line.into(), Kind::Plain)];
    };
    let chars: Vec<char> = line.chars().collect();
    let mut result = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let rest: String = chars[i..].iter().collect();
        if rest.starts_with(lang.line_comment) {
            result.push((rest, Kind::Comment));
            break;
        }
        if chars[i] == '"' || chars[i] == '\'' {
            let quote = chars[i];
            let start = i;
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' {
                    i += 2;
                    continue;
                }
                let done = chars[i] == quote;
                i += 1;
                if done {
                    break;
                }
            }
            result.push((
                chars[start..i.min(chars.len())].iter().collect(),
                Kind::String,
            ));
            continue;
        }
        if chars[i].is_ascii_digit() {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_hexdigit() || ".xob_".contains(chars[i])) {
                i += 1
            }
            result.push((chars[start..i].iter().collect(), Kind::Number));
            continue;
        }
        if chars[i].is_alphabetic() || chars[i] == '_' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1
            }
            let word: String = chars[start..i].iter().collect();
            let kind = if lang.keywords.contains(&word.as_str()) {
                Kind::Keyword
            } else if word.chars().next().is_some_and(char::is_uppercase) {
                Kind::Type
            } else if chars[i..].iter().find(|c| !c.is_whitespace()) == Some(&'(') {
                Kind::Function
            } else {
                Kind::Plain
            };
            result.push((word, kind));
            continue;
        }
        let start = i;
        i += 1;
        while i < chars.len()
            && !chars[i].is_alphanumeric()
            && chars[i] != '_'
            && chars[i] != '"'
            && chars[i] != '\''
        {
            if chars[i..]
                .iter()
                .collect::<String>()
                .starts_with(lang.line_comment)
            {
                break;
            }
            i += 1
        }
        result.push((chars[start..i].iter().collect(), Kind::Plain));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn assert_keyword(file: &str, source: &str, keyword: &str) {
        let language = detect(Path::new(file), None).expect("language should be detected");
        let tokens = highlight(source, Some(&language));
        assert!(
            tokens
                .iter()
                .any(|(text, kind)| text == keyword && *kind == Kind::Keyword),
            "{file}: {keyword} was not highlighted; tokens: {}",
            tokens
                .iter()
                .map(|(text, _)| text.as_str())
                .collect::<String>()
        );
    }

    #[test]
    fn highlights_every_supported_language() {
        assert_keyword("example.py", "def greet(name):", "def");
        assert_keyword("example.rs", "pub fn greet() {}", "fn");
        assert_keyword("Example.java", "public class Example {}", "class");
        assert_keyword("example.lua", "local function greet() end", "function");
        assert_keyword("Example.scala", "object Example extends App", "object");
        assert_keyword("example.c", "static int main(void) {}", "int");
        assert_keyword(
            "example.cpp",
            "class Example { public: void run(); };",
            "class",
        );
    }
}
