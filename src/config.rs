use std::{env, fs, path::PathBuf};

#[derive(Clone)]
pub struct Config {
    pub theme: String,
    pub language: Option<String>,
    pub line_numbers: bool,
    pub wrap: bool,
    pub tab_width: usize,
    pub show_status: bool,
    pub show_shortcuts: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "midnight".into(),
            language: None,
            line_numbers: true,
            wrap: true,
            tab_width: 4,
            show_status: true,
            show_shortcuts: true,
        }
    }
}

impl Config {
    pub fn path() -> Option<PathBuf> {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .map(|p| p.join("thre/config"))
    }

    pub fn load() -> Self {
        let mut c = Self::default();
        let Some(path) = Self::path() else { return c };
        let Ok(text) = fs::read_to_string(path) else {
            return c;
        };
        for raw in text.lines() {
            let line = raw.split('#').next().unwrap_or("").trim();
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim().trim_matches(['"', '\'']);
            match key.trim() {
                "theme" => c.theme = value.into(),
                "language" => {
                    c.language = if value.eq_ignore_ascii_case("auto") {
                        None
                    } else {
                        Some(value.into())
                    }
                }
                "line_numbers" => {
                    if let Some(v) = boolean(value) {
                        c.line_numbers = v
                    }
                }
                "wrap" => {
                    if let Some(v) = boolean(value) {
                        c.wrap = v
                    }
                }
                "tab_width" => {
                    if let Ok(v) = value.parse::<usize>() {
                        c.tab_width = v.clamp(1, 16)
                    }
                }
                "show_status" => {
                    if let Some(v) = boolean(value) {
                        c.show_status = v
                    }
                }
                "show_shortcuts" => {
                    if let Some(v) = boolean(value) {
                        c.show_shortcuts = v
                    }
                }
                _ => {}
            }
        }
        c
    }

    pub fn set_theme(name: &str) -> std::io::Result<()> {
        let path = Self::path()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME is not set"))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let existing = fs::read_to_string(&path).unwrap_or_default();
        let mut found = false;
        let mut output = String::new();
        for line in existing.lines() {
            if line
                .split('=')
                .next()
                .is_some_and(|key| key.trim() == "theme")
            {
                output.push_str(&format!("theme = {name}\n"));
                found = true;
            } else {
                output.push_str(line);
                output.push('\n');
            }
        }
        if !found {
            output.push_str(&format!("theme = {name}\n"));
        }
        fs::write(path, output)
    }
}

fn boolean(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => Some(true),
        "false" | "no" | "off" | "0" => Some(false),
        _ => None,
    }
}
