use crossterm::style::Color;

#[derive(Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub background: Color,
    pub foreground: Color,
    pub muted: Color,
    pub accent: Color,
    pub status_bg: Color,
    pub status_fg: Color,
    pub selection: Color,
    pub keyword: Color,
    pub string: Color,
    pub comment: Color,
    pub number: Color,
    pub type_name: Color,
    pub function: Color,
}

pub fn get(name: &str) -> Theme {
    match name.to_ascii_lowercase().as_str() {
        "graphite" => Theme {
            name: "graphite",
            background: Color::Rgb {
                r: 24,
                g: 24,
                b: 27,
            },
            foreground: Color::Rgb {
                r: 228,
                g: 228,
                b: 231,
            },
            muted: Color::Rgb {
                r: 113,
                g: 113,
                b: 122,
            },
            accent: Color::Rgb {
                r: 192,
                g: 132,
                b: 252,
            },
            status_bg: Color::Rgb {
                r: 39,
                g: 39,
                b: 42,
            },
            status_fg: Color::White,
            selection: Color::Rgb {
                r: 63,
                g: 63,
                b: 70,
            },
            keyword: Color::Rgb {
                r: 216,
                g: 180,
                b: 254,
            },
            string: Color::Rgb {
                r: 134,
                g: 239,
                b: 172,
            },
            comment: Color::Rgb {
                r: 113,
                g: 113,
                b: 122,
            },
            number: Color::Rgb {
                r: 253,
                g: 186,
                b: 116,
            },
            type_name: Color::Rgb {
                r: 103,
                g: 232,
                b: 249,
            },
            function: Color::Rgb {
                r: 147,
                g: 197,
                b: 253,
            },
        },
        "paper" => Theme {
            name: "paper",
            background: Color::Rgb {
                r: 250,
                g: 248,
                b: 242,
            },
            foreground: Color::Rgb {
                r: 42,
                g: 39,
                b: 34,
            },
            muted: Color::Rgb {
                r: 130,
                g: 125,
                b: 115,
            },
            accent: Color::Rgb {
                r: 0,
                g: 95,
                b: 135,
            },
            status_bg: Color::Rgb {
                r: 225,
                g: 221,
                b: 210,
            },
            status_fg: Color::Rgb {
                r: 30,
                g: 30,
                b: 30,
            },
            selection: Color::Rgb {
                r: 210,
                g: 225,
                b: 232,
            },
            keyword: Color::Rgb {
                r: 148,
                g: 35,
                b: 95,
            },
            string: Color::Rgb {
                r: 40,
                g: 115,
                b: 45,
            },
            comment: Color::Rgb {
                r: 125,
                g: 120,
                b: 110,
            },
            number: Color::Rgb {
                r: 180,
                g: 75,
                b: 20,
            },
            type_name: Color::Rgb {
                r: 0,
                g: 100,
                b: 115,
            },
            function: Color::Rgb {
                r: 20,
                g: 75,
                b: 155,
            },
        },
        "ember" => Theme {
            name: "ember",
            background: Color::Rgb {
                r: 29,
                g: 20,
                b: 18,
            },
            foreground: Color::Rgb {
                r: 246,
                g: 232,
                b: 218,
            },
            muted: Color::Rgb {
                r: 145,
                g: 110,
                b: 94,
            },
            accent: Color::Rgb {
                r: 255,
                g: 138,
                b: 76,
            },
            status_bg: Color::Rgb {
                r: 65,
                g: 37,
                b: 29,
            },
            status_fg: Color::Rgb {
                r: 255,
                g: 235,
                b: 215,
            },
            selection: Color::Rgb {
                r: 82,
                g: 49,
                b: 39,
            },
            keyword: Color::Rgb {
                r: 255,
                g: 128,
                b: 105,
            },
            string: Color::Rgb {
                r: 190,
                g: 215,
                b: 110,
            },
            comment: Color::Rgb {
                r: 145,
                g: 110,
                b: 94,
            },
            number: Color::Rgb {
                r: 255,
                g: 190,
                b: 95,
            },
            type_name: Color::Rgb {
                r: 245,
                g: 170,
                b: 105,
            },
            function: Color::Rgb {
                r: 255,
                g: 210,
                b: 135,
            },
        },
        "ocean" => Theme {
            name: "ocean",
            background: Color::Rgb { r: 8, g: 25, b: 35 },
            foreground: Color::Rgb {
                r: 215,
                g: 235,
                b: 238,
            },
            muted: Color::Rgb {
                r: 92,
                g: 132,
                b: 143,
            },
            accent: Color::Rgb {
                r: 45,
                g: 212,
                b: 191,
            },
            status_bg: Color::Rgb {
                r: 13,
                g: 48,
                b: 60,
            },
            status_fg: Color::Rgb {
                r: 224,
                g: 255,
                b: 250,
            },
            selection: Color::Rgb {
                r: 20,
                g: 67,
                b: 78,
            },
            keyword: Color::Rgb {
                r: 94,
                g: 234,
                b: 212,
            },
            string: Color::Rgb {
                r: 163,
                g: 230,
                b: 125,
            },
            comment: Color::Rgb {
                r: 92,
                g: 132,
                b: 143,
            },
            number: Color::Rgb {
                r: 255,
                g: 195,
                b: 105,
            },
            type_name: Color::Rgb {
                r: 103,
                g: 215,
                b: 245,
            },
            function: Color::Rgb {
                r: 135,
                g: 180,
                b: 255,
            },
        },
        _ => Theme {
            name: "midnight",
            background: Color::Rgb {
                r: 11,
                g: 17,
                b: 29,
            },
            foreground: Color::Rgb {
                r: 218,
                g: 226,
                b: 239,
            },
            muted: Color::Rgb {
                r: 91,
                g: 106,
                b: 128,
            },
            accent: Color::Rgb {
                r: 96,
                g: 165,
                b: 250,
            },
            status_bg: Color::Rgb {
                r: 20,
                g: 30,
                b: 48,
            },
            status_fg: Color::Rgb {
                r: 232,
                g: 240,
                b: 252,
            },
            selection: Color::Rgb {
                r: 31,
                g: 48,
                b: 73,
            },
            keyword: Color::Rgb {
                r: 199,
                g: 146,
                b: 234,
            },
            string: Color::Rgb {
                r: 152,
                g: 195,
                b: 121,
            },
            comment: Color::Rgb {
                r: 91,
                g: 106,
                b: 128,
            },
            number: Color::Rgb {
                r: 209,
                g: 154,
                b: 102,
            },
            type_name: Color::Rgb {
                r: 86,
                g: 182,
                b: 194,
            },
            function: Color::Rgb {
                r: 97,
                g: 175,
                b: 239,
            },
        },
    }
}

pub const NAMES: [&str; 5] = ["midnight", "graphite", "paper", "ember", "ocean"];
