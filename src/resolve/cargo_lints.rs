//! Read Cargo's manifest lint configuration after Cargo has validated the TOML.
//! Keep this out of source parsing and the per-function incremental path.

#[derive(Clone, Debug, Default)]
pub(super) struct ManifestLints {
    pub local_cfgs: Vec<String>,
    pub workspace_cfgs: Vec<String>,
    pub inherit: bool,
    pub local_deny: Option<String>,
    pub workspace_deny: Option<String>,
}

#[derive(Debug, PartialEq)]
enum Token {
    Text(String),
    String(String),
    Punct(char),
    Line,
}

#[derive(Debug)]
enum Value {
    Text(String),
    String(String),
    Array(Vec<Value>),
    Table(Vec<(Vec<String>, Value)>),
}

fn string(src: &str, at: &mut usize, quote: u8) -> Result<String, String> {
    let bytes = src.as_bytes();
    let triple = bytes.get(*at..*at + 3).is_some_and(|s| s == [quote; 3]);
    *at += if triple { 3 } else { 1 };
    if triple && bytes.get(*at..*at + 2) == Some(b"\r\n") {
        *at += 2;
    } else if triple && bytes.get(*at) == Some(&b'\n') {
        *at += 1;
    }
    let mut out = String::new();
    while *at < bytes.len() {
        if bytes[*at] == quote {
            let count = bytes[*at..].iter().take_while(|&&b| b == quote).count();
            if !triple || count >= 3 {
                let closing = if triple { 3 } else { 1 };
                if triple {
                    for _ in 3..count.min(5) {
                        out.push(quote as char);
                    }
                }
                *at += if triple { count.min(5) } else { closing };
                return Ok(out);
            }
        }
        if bytes[*at] == b'\\' && quote == b'"' {
            *at += 1;
            let escape = *bytes.get(*at).ok_or("unterminated TOML escape")?;
            *at += 1;
            match escape {
                b'b' => out.push('\u{0008}'),
                b't' => out.push('\t'),
                b'n' => out.push('\n'),
                b'f' => out.push('\u{000c}'),
                b'r' => out.push('\r'),
                b'"' => out.push('"'),
                b'\\' => out.push('\\'),
                b'u' | b'U' => {
                    let count = if escape == b'u' { 4 } else { 8 };
                    let digits = src.get(*at..*at + count).ok_or("short TOML Unicode escape")?;
                    let value = u32::from_str_radix(digits, 16).map_err(|_| "invalid TOML Unicode escape")?;
                    out.push(char::from_u32(value).ok_or("invalid TOML Unicode scalar")?);
                    *at += count;
                }
                b'\r' | b'\n' | b' ' | b'\t' if triple => {
                    while bytes.get(*at).is_some_and(u8::is_ascii_whitespace) {
                        *at += 1;
                    }
                }
                _ => return Err("unsupported TOML string escape in lint configuration".into()),
            }
        } else {
            let c = src[*at..].chars().next().ok_or("unterminated TOML string")?;
            out.push(c);
            *at += c.len_utf8();
        }
    }
    Err("unterminated TOML string".into())
}

fn tokens(src: &str) -> Result<Vec<Token>, String> {
    let mut out = Vec::new();
    let mut at = 0;
    let bytes = src.as_bytes();
    while at < bytes.len() {
        match bytes[at] {
            b'\n' => { out.push(Token::Line); at += 1; }
            b' ' | b'\t' | b'\r' => at += 1,
            b'#' => while bytes.get(at).is_some_and(|&b| b != b'\n') { at += 1; },
            q @ (b'\'' | b'"') => out.push(Token::String(string(src, &mut at, q)?)),
            b @ (b'[' | b']' | b'{' | b'}' | b'=' | b',' | b'.') => {
                out.push(Token::Punct(b as char));
                at += 1;
            }
            _ => {
                let start = at;
                while bytes.get(at).is_some_and(|b| !b.is_ascii_whitespace() && !b"#[]{}=,.\"'".contains(b)) {
                    at += 1;
                }
                if at == start { return Err("invalid TOML token".into()); }
                out.push(Token::Text(src[start..at].to_string()));
            }
        }
    }
    Ok(out)
}

struct Parser {
    tokens: Vec<Token>,
    at: usize,
}

impl Parser {
    fn take(&mut self, c: char) -> bool {
        if self.tokens.get(self.at) == Some(&Token::Punct(c)) {
            self.at += 1;
            true
        } else { false }
    }

    fn lines(&mut self) {
        while self.tokens.get(self.at) == Some(&Token::Line) { self.at += 1; }
    }

    fn key(&mut self) -> Result<Vec<String>, String> {
        let mut key = Vec::new();
        loop {
            match self.tokens.get(self.at) {
                Some(Token::Text(s) | Token::String(s)) => key.push(s.clone()),
                _ => return Err("invalid TOML key in lint configuration".into()),
            }
            self.at += 1;
            if !self.take('.') { return Ok(key); }
        }
    }

    fn value(&mut self) -> Result<Value, String> {
        self.lines();
        if self.take('[') {
            let mut values = Vec::new();
            loop {
                self.lines();
                if self.take(']') { return Ok(Value::Array(values)); }
                values.push(self.value()?);
                self.lines();
                if self.take(']') { return Ok(Value::Array(values)); }
                if !self.take(',') { return Err("invalid TOML array in lint configuration".into()); }
            }
        }
        if self.take('{') {
            let mut fields = Vec::new();
            loop {
                self.lines();
                if self.take('}') { return Ok(Value::Table(fields)); }
                let key = self.key()?;
                if !self.take('=') { return Err("missing TOML assignment".into()); }
                fields.push((key, self.value()?));
                self.lines();
                if self.take('}') { return Ok(Value::Table(fields)); }
                if !self.take(',') { return Err("invalid TOML inline table".into()); }
            }
        }
        let value = match self.tokens.get(self.at) {
            Some(Token::String(s)) => Value::String(s.clone()),
            Some(Token::Text(s)) => {
                let mut text = s.clone();
                self.at += 1;
                while let Some(token) = self.tokens.get(self.at) {
                    match token {
                        Token::Text(s) => { text.push(' '); text.push_str(s); }
                        Token::Punct('.') => text.push('.'),
                        _ => break,
                    }
                    self.at += 1;
                }
                return Ok(Value::Text(text));
            }
            _ => return Err("invalid TOML value in lint configuration".into()),
        };
        self.at += 1;
        Ok(value)
    }
}

fn record(out: &mut ManifestLints, path: &[String], value: Value) -> Result<(), String> {
    if let Value::Table(fields) = value {
        for (key, v) in fields {
            let mut nested = path.to_vec();
            nested.extend(key);
            record(out, &nested, v)?;
        }
        return Ok(());
    }
    let path: Vec<&str> = path.iter().map(String::as_str).collect();
    if path == ["lints", "workspace"] {
        out.inherit = matches!(value, Value::Text(ref s) if s == "true");
        return Ok(());
    }
    let (rest, workspace) = match path.as_slice() {
        ["lints", "rust", rest @ ..] => (rest, false),
        ["workspace", "lints", "rust", rest @ ..] => (rest, true),
        _ => return Ok(()),
    };
    if rest == ["unexpected_cfgs", "check-cfg"] {
        let Value::Array(values) = value else { return Err("check-cfg must be an array of strings".into()); };
        let dest = if workspace { &mut out.workspace_cfgs } else { &mut out.local_cfgs };
        for value in values {
            let Value::String(value) = value else { return Err("check-cfg must contain strings".into()); };
            dest.push(value);
        }
    } else if let [name] | [name, "level"] = rest {
        if *name != "unsafe_code" && matches!(value, Value::String(ref s) if s == "deny" || s == "forbid") {
            let dest = if workspace { &mut out.workspace_deny } else { &mut out.local_deny };
            dest.get_or_insert_with(|| name.to_string());
        }
    }
    Ok(())
}

/// Cargo has already accepted the manifest. We only interpret lint fields;
/// unrelated scalar values, including dates and floats, stay opaque.
pub(super) fn parse(src: &str) -> Result<ManifestLints, String> {
    let mut p = Parser { tokens: tokens(src)?, at: 0 };
    let mut out = ManifestLints::default();
    let mut section = Vec::new();
    while p.at < p.tokens.len() {
        p.lines();
        if p.at == p.tokens.len() { break; }
        if p.take('[') {
            let array = p.take('[');
            section = p.key()?;
            if !p.take(']') || array && !p.take(']') { return Err("invalid TOML table header".into()); }
            if array { section.insert(0, "[[array]]".into()); }
        } else {
            let mut key = section.clone();
            key.extend(p.key()?);
            if !p.take('=') { return Err("missing TOML assignment".into()); }
            let relevant = key.first().is_some_and(|s| s == "lints")
                || key.first().is_some_and(|s| s == "workspace")
                    && (key.len() == 1 || key.get(1).is_some_and(|s| s == "lints"));
            if relevant {
                let value = p.value()?;
                record(&mut out, &key, value)?;
            } else {
                // Skip a complete value, respecting nested delimiters and
                // strings. No allocation of unrelated arrays or inline tables.
                let mut depth = 0usize;
                while let Some(token) = p.tokens.get(p.at) {
                    match token {
                        Token::Line if depth == 0 => break,
                        Token::Punct('[' | '{') => depth += 1,
                        Token::Punct(']' | '}') => depth = depth.saturating_sub(1),
                        _ => {}
                    }
                    p.at += 1;
                }
            }
        }
        while p.tokens.get(p.at).is_some_and(|t| *t != Token::Line) { p.at += 1; }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_cfg_is_a_declaration_and_toml_shapes_agree() {
        for src in [
            "[lints.rust]\nunexpected_cfgs = { level = 'warn', check-cfg = ['cfg(checked_build)', 'cfg(mode, values(\"small\"))'] }\n",
            "[lints.rust.unexpected_cfgs]\nlevel = 'warn'\ncheck-cfg = [\n'cfg(checked_build)', # a comment with ] and \"\n'cfg(mode, values(\"small\"))',\n]\n",
            "lints.rust.unexpected_cfgs = { check-cfg = ['cfg(checked_build)', 'cfg(mode, values(\"small\"))'], level = 'warn' }\n",
            "[lints]\nrust = { unexpected_cfgs = { check-cfg = ['cfg(checked_build)', 'cfg(mode, values(\"small\"))'] } }\n",
        ] {
            let got = parse(src).unwrap();
            assert_eq!(got.local_cfgs, ["cfg(checked_build)", "cfg(mode, values(\"small\"))"]);
            assert!(!got.inherit);
            assert!(got.local_deny.is_none());
        }
    }

    #[test]
    fn workspace_opt_in_and_other_tables_remain_separate() {
        let got = parse("[workspace.lints.rust]\nunexpected_cfgs = { level = 'warn', check-cfg = ['cfg(workspace_mode)'] }\n[lints]\nworkspace = true\n[package.metadata]\ncheck-cfg = ['cfg(not_a_lint)']\n").unwrap();
        assert!(got.inherit);
        assert_eq!(got.workspace_cfgs, ["cfg(workspace_mode)"]);
        assert!(got.local_cfgs.is_empty());
    }

    #[test]
    fn escaped_strings_comments_and_denied_levels_are_not_lost() {
        let got = parse("[lints.\"rust\".unexpected_cfgs]\nlevel = \"deny\"\ncheck-cfg = [\"cfg(mode, values(\\\"a#b\\\"))\"] # ignored\n").unwrap();
        assert_eq!(got.local_deny.as_deref(), Some("unexpected_cfgs"));
        assert_eq!(got.local_cfgs, ["cfg(mode, values(\"a#b\"))"]);
        assert!(parse("[lints.rust.unexpected_cfgs]\ncheck-cfg = [false]\n").is_err());
    }
}
