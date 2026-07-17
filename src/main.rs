mod app;
mod config;
mod syntax;
mod theme;

use std::{env, fs, io, path::PathBuf, process::ExitCode};

use app::App;
use config::Config;

const HELP: &str = r#"thre — a comfortable terminal file reader

Usage: thre [OPTIONS] <FILE ...>

Options:
  -t, --theme <NAME>       Use a theme; without a file, set the default
  -l, --language <LANG>    Override syntax detection
      --no-line-numbers    Hide line numbers
      --no-wrap            Disable soft wrapping
      --list-themes        Show available themes
  -h, --help               Show this help
  -V, --version            Show version

Inside the reader, press F1 or ? to see every shortcut."#;

#[derive(Default)]
struct Args {
    files: Vec<PathBuf>,
    theme: Option<String>,
    language: Option<String>,
    line_numbers: Option<bool>,
    wrap: Option<bool>,
}

fn parse_args() -> Result<Args, String> {
    let mut out = Args::default();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{HELP}");
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("thre {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "--list-themes" => {
                println!("midnight\ngraphite\npaper\nember\nocean");
                std::process::exit(0);
            }
            "-t" | "--theme" => out.theme = Some(args.next().ok_or("--theme needs a name")?),
            "-l" | "--language" => {
                out.language = Some(args.next().ok_or("--language needs a name")?)
            }
            "--no-line-numbers" => out.line_numbers = Some(false),
            "--no-wrap" => out.wrap = Some(false),
            "--" => {
                out.files.extend(args.map(PathBuf::from));
                break;
            }
            _ if arg.starts_with('-') => return Err(format!("unknown option: {arg}")),
            _ => out.files.push(PathBuf::from(arg)),
        }
    }
    Ok(out)
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args().map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    if let Some(name) = args.theme.as_deref() {
        if !theme::NAMES.contains(&name.to_ascii_lowercase().as_str()) {
            return Err(format!("unknown theme: {name} (try `thre --list-themes`)").into());
        }
        if args.files.is_empty() {
            Config::set_theme(name)?;
            println!("Default theme set to {name}");
            return Ok(());
        }
    }
    if args.files.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no file supplied (try `thre --help`)",
        )
        .into());
    }
    let new_buffer = false;
    let mut files = args.files.into_iter();
    let path = files
        .next()
        .unwrap_or_else(|| PathBuf::from("untitled.txt"));
    let bytes = match fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() => fs::read(&path)?,
        Ok(_) => return Err(format!("{} is not a regular file", path.display()).into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(error.into()),
    };
    if bytes.contains(&0) {
        return Err("binary files are not supported".into());
    }
    let content = String::from_utf8_lossy(&bytes).replace("\r\n", "\n");

    let mut config = Config::load();
    if let Some(v) = args.theme {
        config.theme = v;
    }
    if let Some(v) = args.language {
        config.language = Some(v);
    }
    if let Some(v) = args.line_numbers {
        config.line_numbers = v;
    }
    if let Some(v) = args.wrap {
        config.wrap = v;
    }
    let mut app = App::new(path, content, bytes.len(), config, new_buffer)?;
    for path in files {
        app.add_startup_file(path)?;
    }
    app.run()?;
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("thre: {error}");
            ExitCode::FAILURE
        }
    }
}
