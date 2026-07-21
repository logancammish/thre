use std::{
    cmp, fs,
    io::{self, stdout, Write},
    path::PathBuf,
    time::Duration,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseButton, MouseEvent, MouseEventKind,
    },
    execute, queue,
    style::{
        Attribute, Color, Colored, Print, ResetColor, SetAttribute, SetBackgroundColor,
        SetForegroundColor,
    },
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{
    config::Config,
    syntax::{self, Kind, Language},
    theme::{self, Theme},
};

#[derive(Clone)]
enum Prompt {
    Search(String),
    Goto(String),
    SaveAs(String),
    Open(String),
    Rename(String),
    Quit,
}

#[derive(Clone)]
pub struct App {
    path: PathBuf,
    lines: Vec<String>,
    original_size: usize,
    config: Config,
    theme: Theme,
    language: Option<Language>,
    row: usize,
    col: usize,
    top: usize,
    left: usize,
    dirty: bool,
    message: String,
    prompt: Option<Prompt>,
    help: bool,
    quit: bool,
    matches: Vec<(usize, usize)>,
    match_index: usize,
    new_buffer: bool,
    tabs: Vec<App>,
    active_tab: usize,
    selection_anchor: Option<(usize, usize)>,
    read_markdown: bool,
}

struct Terminal;
impl Terminal {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, EnableMouseCapture, Hide)?;
        Ok(Self)
    }
}
impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = execute!(
            stdout(),
            DisableMouseCapture,
            Show,
            LeaveAlternateScreen,
            ResetColor
        );
        let _ = terminal::disable_raw_mode();
    }
}

impl App {
    pub fn new(
        path: PathBuf,
        content: String,
        size: usize,
        config: Config,
        new_buffer: bool,
        read_markdown: bool,
    ) -> io::Result<Self> {
        let theme = theme::get(&config.theme);
        let language = syntax::detect(&path, config.language.as_deref());
        let mut lines: Vec<String> = content.split('\n').map(str::to_owned).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        Ok(Self {
            path,
            lines,
            original_size: size,
            config,
            theme,
            language,
            row: 0,
            col: 0,
            top: 0,
            left: 0,
            dirty: false,
            message: "Ready".into(),
            prompt: None,
            help: false,
            quit: false,
            matches: vec![],
            match_index: 0,
            new_buffer,
            tabs: Vec::new(),
            active_tab: 0,
            selection_anchor: None,
            read_markdown,
        })
    }

    pub fn run(mut self) -> io::Result<()> {
        // thre is an explicitly themed full-screen application. Crossterm otherwise
        // silently strips every theme and syntax color when NO_COLOR is inherited.
        Colored::set_ansi_color_disabled(false);
        let _guard = Terminal::enter()?;
        while !self.quit {
            self.draw()?;
            if event::poll(Duration::from_millis(250))? {
                match event::read()? {
                    Event::Key(k) if k.kind == event::KeyEventKind::Press => self.key(k)?,
                    Event::Mouse(mouse) => self.mouse(mouse),
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn mouse(&mut self, mouse: MouseEvent) {
        if self.help || self.prompt.is_some() {
            return;
        }
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.selection_anchor = None;
                if mouse.row == 0 {
                    self.click_tab(mouse.column as usize);
                } else {
                    self.click_content(mouse.column as usize, mouse.row.saturating_sub(1) as usize);
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if self.selection_anchor.is_none() {
                    self.selection_anchor = Some((self.row, self.col));
                }
                self.click_content(mouse.column as usize, mouse.row.saturating_sub(1) as usize);
            }
            MouseEventKind::ScrollDown => self.scroll(3),
            MouseEventKind::ScrollUp => self.scroll(-3),
            _ => {}
        }
    }

    fn click_tab(&mut self, x: usize) {
        self.ensure_current_tab();
        let mut start = 0;
        for (index, tab) in self.tabs.iter().enumerate() {
            let width = tab_label(tab, index == self.active_tab).width();
            if x >= start && x < start + width {
                let direction = index as i32 - self.active_tab as i32;
                if direction != 0 {
                    self.switch_tab(direction);
                }
                return;
            }
            start += width;
        }
    }

    fn click_content(&mut self, x: usize, y: usize) {
        let (width, height) = terminal::size().unwrap_or((80, 24));
        let content_height = height.saturating_sub(3) as usize;
        if y >= content_height {
            return;
        }
        let digits = self.lines.len().to_string().len();
        let gutter = if self.config.line_numbers {
            digits + 2
        } else {
            1
        };
        let available = (width as usize).saturating_sub(gutter).max(1);
        let mut visual_y = 0;
        for row in self.top..self.lines.len() {
            let starts = if self.config.wrap {
                wrap_starts(&self.lines[row], available)
            } else {
                vec![self.left]
            };
            let rows = starts.len();
            if y < visual_y + rows {
                let segment = if self.config.wrap { y - visual_y } else { 0 };
                let start = starts[segment];
                let target_cell = x.saturating_sub(gutter);
                self.row = row;
                let tail: String = self.lines[row].chars().skip(start).collect();
                self.col = start + cell_to_char(&tail, target_cell);
                self.keep_visible();
                return;
            }
            visual_y += rows;
            if visual_y > y {
                break;
            }
        }
    }

    fn scroll(&mut self, amount: i32) {
        if amount > 0 {
            let delta = amount as usize;
            let actual = delta.min(self.lines.len().saturating_sub(1).saturating_sub(self.top));
            self.top += actual;
            self.row = (self.row + actual).min(self.lines.len() - 1);
        } else {
            let actual = ((-amount) as usize).min(self.top);
            self.top -= actual;
            self.row = self.row.saturating_sub(actual);
        }
        self.col = self.col.min(self.line_len());
    }

    pub fn add_startup_file(&mut self, path: PathBuf) -> io::Result<()> {
        self.open_tab(path)
    }

    fn key(&mut self, key: KeyEvent) -> io::Result<()> {
        if self.help {
            self.help = false;
            return Ok(());
        }
        if self.prompt.is_some() {
            return self.prompt_key(key);
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let before = (self.row, self.col);
        match (key.code, ctrl) {
            (KeyCode::Char('a'), true) => {
                self.selection_anchor = Some((0, 0));
                self.row = self.lines.len() - 1;
                self.col = self.line_len();
                self.message = "Selected all".into();
            }
            (KeyCode::Char('c'), true) => self.copy_selection()?,
            (KeyCode::Char('x'), true) => self.quit = true,
            (KeyCode::Char('q'), true) => {
                if self.dirty {
                    self.prompt = Some(Prompt::Quit)
                } else {
                    self.quit = true
                }
            }
            (KeyCode::Char('s'), true) => self.request_save()?,
            (KeyCode::Char('o'), true) => self.prompt = Some(Prompt::Open(String::new())),
            (KeyCode::Char('n'), true) => self.new_tab()?,
            (KeyCode::Tab, true) => self.switch_tab(1),
            (KeyCode::BackTab, true) => self.switch_tab(-1),
            (KeyCode::Char('f'), true) => self.prompt = Some(Prompt::Search(String::new())),
            (KeyCode::Char('g'), true) => self.prompt = Some(Prompt::Goto(String::new())),
            (KeyCode::Char('l'), true) => self.config.line_numbers = !self.config.line_numbers,
            (KeyCode::Char('w'), true) => {
                self.config.wrap = !self.config.wrap;
                self.left = 0;
                self.message = format!("Soft wrap {}", if self.config.wrap { "on" } else { "off" });
            }
            (KeyCode::Char('t'), true) | (KeyCode::F(2), _) => self.next_theme(),
            (KeyCode::F(3), _) => {
                self.prompt = Some(Prompt::Rename(self.path.to_string_lossy().into_owned()))
            }
            (KeyCode::F(1), _) | (KeyCode::Char('?'), false) => self.help = true,
            (KeyCode::Up, _) => self.move_vertical(-1),
            (KeyCode::Down, _) => self.move_vertical(1),
            (KeyCode::Left, _) => self.move_left(),
            (KeyCode::Right, _) => self.move_right(),
            (KeyCode::Home, _) => self.col = 0,
            (KeyCode::End, _) => self.col = self.line_len(),
            (KeyCode::PageUp, _) => {
                let h = self.content_height();
                self.row = self.row.saturating_sub(h);
                self.col = cmp::min(self.col, self.line_len());
            }
            (KeyCode::PageDown, _) => {
                let h = self.content_height();
                self.row = cmp::min(self.row + h, self.lines.len() - 1);
                self.col = cmp::min(self.col, self.line_len());
            }
            (KeyCode::Enter, _) => self.newline(),
            (KeyCode::Backspace, _) => self.backspace(),
            (KeyCode::Delete, _) => self.delete(),
            (KeyCode::Tab, _) => {
                for _ in 0..self.config.tab_width {
                    self.insert(' ')
                }
            }
            (KeyCode::Char(c), false) | (KeyCode::Char(c), true) if !ctrl => self.insert(c),
            _ => {}
        }
        let movement = matches!(
            key.code,
            KeyCode::Up
                | KeyCode::Down
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::Home
                | KeyCode::End
                | KeyCode::PageUp
                | KeyCode::PageDown
        );
        if movement {
            if shift {
                if self.selection_anchor.is_none() {
                    self.selection_anchor = Some(before);
                }
            } else {
                self.selection_anchor = None;
            }
        }
        self.keep_visible();
        Ok(())
    }

    fn prompt_key(&mut self, key: KeyEvent) -> io::Result<()> {
        match self.prompt.take().unwrap() {
            Prompt::Quit => match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.save()?;
                    self.quit = true
                }
                KeyCode::Char('n') | KeyCode::Char('N') => self.quit = true,
                _ => self.message = "Quit cancelled".into(),
            },
            Prompt::Search(mut s) => match key.code {
                KeyCode::Esc => {}
                KeyCode::Enter => self.search(&s),
                KeyCode::Backspace => {
                    s.pop();
                    self.prompt = Some(Prompt::Search(s))
                }
                KeyCode::Char(c) => {
                    s.push(c);
                    self.prompt = Some(Prompt::Search(s))
                }
                _ => self.prompt = Some(Prompt::Search(s)),
            },
            Prompt::Goto(mut s) => match key.code {
                KeyCode::Esc => {}
                KeyCode::Enter => {
                    if let Ok(n) = s.parse::<usize>() {
                        self.row = n.saturating_sub(1).min(self.lines.len() - 1);
                        self.col = self.col.min(self.line_len());
                        self.keep_visible()
                    }
                }
                KeyCode::Backspace => {
                    s.pop();
                    self.prompt = Some(Prompt::Goto(s))
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    s.push(c);
                    self.prompt = Some(Prompt::Goto(s))
                }
                _ => self.prompt = Some(Prompt::Goto(s)),
            },
            Prompt::SaveAs(mut s) => match key.code {
                KeyCode::Esc => self.message = "Save cancelled".into(),
                KeyCode::Enter if !s.trim().is_empty() => {
                    self.path = PathBuf::from(s.trim());
                    self.new_buffer = false;
                    self.save()?;
                }
                KeyCode::Enter => self.prompt = Some(Prompt::SaveAs(s)),
                KeyCode::Backspace => {
                    s.pop();
                    self.prompt = Some(Prompt::SaveAs(s))
                }
                KeyCode::Char(c) => {
                    s.push(c);
                    self.prompt = Some(Prompt::SaveAs(s))
                }
                _ => self.prompt = Some(Prompt::SaveAs(s)),
            },
            Prompt::Open(mut s) => match key.code {
                KeyCode::Esc => self.message = "Open cancelled".into(),
                KeyCode::Enter if !s.trim().is_empty() => self.open_tab(PathBuf::from(s.trim()))?,
                KeyCode::Enter => self.prompt = Some(Prompt::Open(s)),
                KeyCode::Backspace => {
                    s.pop();
                    self.prompt = Some(Prompt::Open(s))
                }
                KeyCode::Char(c) => {
                    s.push(c);
                    self.prompt = Some(Prompt::Open(s))
                }
                _ => self.prompt = Some(Prompt::Open(s)),
            },
            Prompt::Rename(mut s) => match key.code {
                KeyCode::Esc => self.message = "Rename cancelled".into(),
                KeyCode::Enter if !s.trim().is_empty() => self.rename(PathBuf::from(s.trim()))?,
                KeyCode::Enter => self.prompt = Some(Prompt::Rename(s)),
                KeyCode::Backspace => {
                    s.pop();
                    self.prompt = Some(Prompt::Rename(s))
                }
                KeyCode::Char(c) => {
                    s.push(c);
                    self.prompt = Some(Prompt::Rename(s))
                }
                _ => self.prompt = Some(Prompt::Rename(s)),
            },
        }
        Ok(())
    }

    fn insert(&mut self, c: char) {
        self.delete_selection();
        let byte = char_to_byte(&self.lines[self.row], self.col);
        self.lines[self.row].insert(byte, c);
        self.col += 1;
        self.changed()
    }
    fn newline(&mut self) {
        self.delete_selection();
        let byte = char_to_byte(&self.lines[self.row], self.col);
        let tail = self.lines[self.row].split_off(byte);
        self.row += 1;
        self.col = 0;
        self.lines.insert(self.row, tail);
        self.changed()
    }
    fn backspace(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.col > 0 {
            let a = char_to_byte(&self.lines[self.row], self.col - 1);
            let b = char_to_byte(&self.lines[self.row], self.col);
            self.lines[self.row].replace_range(a..b, "");
            self.col -= 1;
            self.changed()
        } else if self.row > 0 {
            let line = self.lines.remove(self.row);
            self.row -= 1;
            self.col = self.line_len();
            self.lines[self.row].push_str(&line);
            self.changed()
        }
    }
    fn delete(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.col < self.line_len() {
            let a = char_to_byte(&self.lines[self.row], self.col);
            let b = char_to_byte(&self.lines[self.row], self.col + 1);
            self.lines[self.row].replace_range(a..b, "");
            self.changed()
        } else if self.row + 1 < self.lines.len() {
            let next = self.lines.remove(self.row + 1);
            self.lines[self.row].push_str(&next);
            self.changed()
        }
    }
    fn changed(&mut self) {
        self.dirty = true;
        self.message = "Modified".into();
        self.matches.clear()
    }
    fn selection(&self) -> Option<((usize, usize), (usize, usize))> {
        let anchor = self.selection_anchor?;
        let cursor = (self.row, self.col);
        if anchor == cursor {
            return None;
        }
        Some(if anchor < cursor {
            (anchor, cursor)
        } else {
            (cursor, anchor)
        })
    }
    fn selected(&self, row: usize, col: usize) -> bool {
        self.selection()
            .is_some_and(|(start, end)| (row, col) >= start && (row, col) < end)
    }
    fn delete_selection(&mut self) -> bool {
        let Some(((sr, sc), (er, ec))) = self.selection() else {
            return false;
        };
        if sr == er {
            let a = char_to_byte(&self.lines[sr], sc);
            let b = char_to_byte(&self.lines[sr], ec);
            self.lines[sr].replace_range(a..b, "");
        } else {
            let prefix: String = self.lines[sr].chars().take(sc).collect();
            let suffix: String = self.lines[er].chars().skip(ec).collect();
            self.lines.splice(sr..=er, [format!("{prefix}{suffix}")]);
        }
        self.row = sr;
        self.col = sc;
        self.selection_anchor = None;
        self.changed();
        true
    }
    fn copy_selection(&mut self) -> io::Result<()> {
        let Some(((sr, sc), (er, ec))) = self.selection() else {
            self.message = "Nothing selected".into();
            return Ok(());
        };
        let text = if sr == er {
            self.lines[sr].chars().skip(sc).take(ec - sc).collect()
        } else {
            let mut parts = Vec::with_capacity(er - sr + 1);
            parts.push(self.lines[sr].chars().skip(sc).collect::<String>());
            parts.extend(self.lines[sr + 1..er].iter().cloned());
            parts.push(self.lines[er].chars().take(ec).collect());
            parts.join("\n")
        };
        // OSC 52 is supported by the major terminal emulators and works without
        // adding a platform-specific clipboard dependency.
        write!(stdout(), "\x1b]52;c;{}\x07", base64(text.as_bytes()))?;
        stdout().flush()?;
        self.message = format!("Copied {} characters", text.chars().count());
        Ok(())
    }
    fn move_left(&mut self) {
        if self.col > 0 {
            self.col -= 1
        } else if self.row > 0 {
            self.row -= 1;
            self.col = self.line_len()
        }
    }
    fn move_right(&mut self) {
        if self.col < self.line_len() {
            self.col += 1
        } else if self.row + 1 < self.lines.len() {
            self.row += 1;
            self.col = 0
        }
    }
    fn move_vertical(&mut self, d: i32) {
        self.row = if d < 0 {
            self.row.saturating_sub(1)
        } else {
            cmp::min(self.row + 1, self.lines.len() - 1)
        };
        self.col = cmp::min(self.col, self.line_len())
    }
    fn line_len(&self) -> usize {
        self.lines[self.row].chars().count()
    }
    fn content_height(&self) -> usize {
        terminal::size()
            .map(|(_, h)| h.saturating_sub(2) as usize)
            .unwrap_or(20)
            .max(1)
    }
    fn keep_visible(&mut self) {
        let h = self.content_height();
        if self.row < self.top {
            self.top = self.row
        }
        if self.row >= self.top + h {
            self.top = self.row - h + 1
        }
        let width = terminal::size().map(|(w, _)| w as usize).unwrap_or(80);
        if !self.config.wrap {
            if self.col < self.left {
                self.left = self.col
            }
            if self.col >= self.left + width.saturating_sub(10) {
                self.left = self.col - width.saturating_sub(10) + 1
            }
        }
    }

    fn save(&mut self) -> io::Result<()> {
        let text = self.lines.join("\n");
        fs::write(&self.path, text.as_bytes())?;
        self.original_size = text.len();
        self.dirty = false;
        self.message = format!("Saved {}", self.path.display());
        Ok(())
    }
    fn request_save(&mut self) -> io::Result<()> {
        if self.new_buffer {
            self.prompt = Some(Prompt::SaveAs(String::new()));
            Ok(())
        } else {
            self.save()
        }
    }
    fn snapshot(&self) -> Self {
        let mut copy = self.clone();
        copy.tabs.clear();
        copy
    }
    fn ensure_current_tab(&mut self) {
        if self.tabs.is_empty() {
            self.tabs.push(self.snapshot());
            self.active_tab = 0;
        } else {
            self.tabs[self.active_tab] = self.snapshot();
        }
    }
    fn switch_tab(&mut self, direction: i32) {
        self.ensure_current_tab();
        if self.tabs.len() < 2 {
            return;
        }
        let next = (self.active_tab as i32 + direction).rem_euclid(self.tabs.len() as i32) as usize;
        let tabs = std::mem::take(&mut self.tabs);
        let mut target = tabs[next].clone();
        target.tabs = tabs;
        target.active_tab = next;
        *self = target;
        self.message = format!("Tab {} of {}", next + 1, self.tabs.len());
    }
    fn new_tab(&mut self) -> io::Result<()> {
        self.ensure_current_tab();
        let mut tab = App::new(
            PathBuf::from("untitled.txt"),
            String::new(),
            0,
            self.config.clone(),
            true,
            self.read_markdown,
        )?;
        tab.theme = self.theme.clone();
        self.tabs.push(tab.snapshot());
        self.active_tab = self.tabs.len() - 2;
        self.switch_tab(1);
        Ok(())
    }
    fn open_tab(&mut self, path: PathBuf) -> io::Result<()> {
        let bytes = fs::read(&path)?;
        if bytes.contains(&0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "binary files are not supported",
            ));
        }
        self.ensure_current_tab();
        let content = String::from_utf8_lossy(&bytes).replace("\r\n", "\n");
        let mut tab = App::new(
            path,
            content,
            bytes.len(),
            self.config.clone(),
            false,
            self.read_markdown,
        )?;
        tab.theme = self.theme.clone();
        self.tabs.push(tab.snapshot());
        self.active_tab = self.tabs.len() - 2;
        self.switch_tab(1);
        Ok(())
    }
    fn rename(&mut self, path: PathBuf) -> io::Result<()> {
        if !self.new_buffer && self.path != path {
            fs::rename(&self.path, &path)?;
        }
        self.path = path;
        self.new_buffer = false;
        self.language = syntax::detect(&self.path, self.config.language.as_deref());
        self.message = format!("Renamed to {}", self.path.display());
        Ok(())
    }
    fn search(&mut self, query: &str) {
        self.matches.clear();
        if query.is_empty() {
            self.message = "Search cancelled".into();
            return;
        }
        for (r, line) in self.lines.iter().enumerate() {
            let mut start = 0;
            while let Some(p) = line[start..].find(query) {
                self.matches.push((r, line[..start + p].chars().count()));
                start += p + query.len()
            }
        }
        if let Some(&(r, c)) = self.matches.first() {
            self.match_index = 0;
            self.row = r;
            self.col = c;
            self.keep_visible();
            self.message = format!("1 of {} matches", self.matches.len())
        } else {
            self.message = format!("No matches for “{query}”")
        }
    }
    fn next_theme(&mut self) {
        let names = theme::names();
        let i = names
            .iter()
            .position(|name| name == &self.theme.name)
            .unwrap_or(0);
        self.theme = theme::get(&names[(i + 1) % names.len()]);
        self.message = format!("Theme: {}", self.theme.name)
    }

    fn draw(&mut self) -> io::Result<()> {
        self.ensure_current_tab();
        let (w, h) = terminal::size()?;
        let mut out = stdout();
        queue!(
            out,
            Hide,
            SetBackgroundColor(self.theme.background),
            SetForegroundColor(self.theme.foreground),
            Clear(ClearType::All)
        )?;
        if self.help {
            self.draw_help(&mut out, w, h)?;
            out.flush()?;
            return Ok(());
        }
        let mut tab_text = String::new();
        for (index, tab) in self.tabs.iter().enumerate() {
            tab_text.push_str(&tab_label(tab, index == self.active_tab));
        }
        queue!(
            out,
            MoveTo(0, 0),
            SetBackgroundColor(self.theme.status_bg),
            SetForegroundColor(self.theme.foreground),
            Print(fit(&tab_text, w as usize))
        )?;
        let content_start = 1u16;
        let status_rows = if self.config.show_status { 1 } else { 0 };
        let shortcut_rows = if self.config.show_shortcuts { 1 } else { 0 };
        let content_h = h.saturating_sub(status_rows + shortcut_rows + content_start) as usize;
        let digits = self.lines.len().to_string().len();
        let gutter = if self.config.line_numbers {
            digits + 2
        } else {
            1
        };
        let available = (w as usize).saturating_sub(gutter).max(1);
        let mut screen_y = 0usize;
        let mut cursor_y = 0usize;
        for r in self.top..self.lines.len() {
            let starts: Vec<usize> = if self.config.wrap {
                wrap_starts(&self.lines[r], available)
            } else {
                vec![self.left]
            };
            for (start_i, start) in starts.into_iter().enumerate() {
                if screen_y >= content_h {
                    break;
                }
                if r == self.row && self.col >= start && self.col <= start + available {
                    cursor_y = screen_y
                }
                queue!(
                    out,
                    MoveTo(0, content_start + screen_y as u16),
                    SetBackgroundColor(self.theme.background)
                )?;
                if self.config.line_numbers {
                    let label = if start_i == 0 {
                        format!("{:>digits$}  ", r + 1, digits = digits)
                    } else {
                        " ".repeat(gutter)
                    };
                    queue!(
                        out,
                        SetForegroundColor(if r == self.row {
                            self.theme.accent
                        } else {
                            self.theme.muted
                        }),
                        Print(label)
                    )?
                } else {
                    queue!(out, Print(" "))?
                }
                let line: String = self.lines[r].chars().skip(start).take(available).collect();
                if self.read_markdown {
                    let mut col = start;
                    for span in crate::markdown::spans(&line) {
                        queue!(
                            out,
                            SetAttribute(span.attribute),
                            SetForegroundColor(self.theme.foreground)
                        )?;
                        for ch in span.text.chars() {
                            let bg = if self.selected(r, col) {
                                self.theme.selection
                            } else {
                                self.theme.background
                            };
                            queue!(out, SetBackgroundColor(bg), Print(ch))?;
                            col += 1;
                        }
                    }
                    queue!(out, SetAttribute(Attribute::Reset))?;
                } else {
                    let mut col = start;
                    for (text, kind) in syntax::highlight(&line, self.language.as_ref()) {
                        queue!(out, SetForegroundColor(self.color(kind)))?;
                        for ch in text.chars() {
                            let bg = if self.selected(r, col) {
                                self.theme.selection
                            } else {
                                self.theme.background
                            };
                            queue!(out, SetBackgroundColor(bg), Print(ch))?;
                            col += 1;
                        }
                    }
                }
                screen_y += 1
            }
            if screen_y >= content_h {
                break;
            }
        }
        if self.config.show_status {
            let y = h - shortcut_rows - 1;
            let lang = if self.read_markdown {
                "Markdown"
            } else {
                self.language
                    .as_ref()
                    .map(|l| l.name.as_str())
                    .unwrap_or("Plain text")
            };
            let dirty = if self.dirty { " • modified" } else { "" };
            let left = format!(
                "  {}{}",
                self.path.file_name().unwrap_or_default().to_string_lossy(),
                dirty
            );
            let right = format!(
                "{}  Ln {}, Col {}  {}%  ",
                lang,
                self.row + 1,
                self.col + 1,
                (self.row + 1) * 100 / self.lines.len().max(1)
            );
            bar(
                &mut out,
                y,
                w,
                &left,
                &right,
                self.theme.status_bg,
                self.theme.status_fg,
            )?
        }
        if self.config.show_shortcuts {
            let y = h - 1;
            let text = match &self.prompt {
                Some(Prompt::Search(s)) => format!(" Search: {s}_   Enter find   Esc cancel"),
                Some(Prompt::Goto(s)) => format!(" Go to line: {s}_   Enter go   Esc cancel"),
                Some(Prompt::SaveAs(s)) => {
                    format!(" Save as: {s}_   Enter save   Esc cancel")
                }
                Some(Prompt::Open(s)) => format!(" Open file: {s}_   Enter open   Esc cancel"),
                Some(Prompt::Rename(s)) => {
                    format!(" Rename to: {s}_   Enter rename   Esc cancel")
                }
                Some(Prompt::Quit) => {
                    " Save changes before quitting?   Y yes   N no   any other key cancel".into()
                }
                None => format!(
                    " ^S Save   ^Q Safe quit   ^X Exit now   ^O Open   ^N New   ^Tab Next   F1 Help   {}",
                    self.message
                ),
            };
            queue!(
                out,
                MoveTo(0, y),
                SetBackgroundColor(self.theme.selection),
                SetForegroundColor(self.theme.foreground),
                Print(fit(&text, w as usize))
            )?
        }
        if self.prompt.is_none() {
            let cursor_start = if self.config.wrap {
                wrap_starts(&self.lines[self.row], available)
                    .into_iter()
                    .take_while(|start| *start <= self.col)
                    .last()
                    .unwrap_or(0)
            } else {
                self.left
            };
            let before_cursor: String = self.lines[self.row]
                .chars()
                .skip(cursor_start)
                .take(self.col.saturating_sub(cursor_start))
                .collect();
            let visual_col = before_cursor.width();
            let x = (gutter + visual_col).min(w.saturating_sub(1) as usize) as u16;
            queue!(
                out,
                MoveTo(
                    x,
                    content_start + cursor_y.min(content_h.saturating_sub(1)) as u16
                ),
                Show
            )?
        }
        queue!(out, ResetColor)?;
        out.flush()
    }
    fn color(&self, k: Kind) -> Color {
        match k {
            Kind::Plain => self.theme.foreground,
            Kind::Keyword => self.theme.keyword,
            Kind::String => self.theme.string,
            Kind::Comment => self.theme.comment,
            Kind::Number => self.theme.number,
            Kind::Type => self.theme.type_name,
            Kind::Function => self.theme.function,
        }
    }
    fn draw_help(&self, out: &mut impl Write, w: u16, h: u16) -> io::Result<()> {
        let lines = [
            "thre shortcuts",
            "",
            "Navigation",
            "  Arrows        Move cursor",
            "  Home / End    Start / end of line",
            "  Page Up/Down  Move one screen",
            "",
            "Editing",
            "  Type          Insert text",
            "  Enter         New line",
            "  Backspace     Remove previous character",
            "  Delete        Remove next character",
            "  Ctrl+A        Select all text",
            "  Ctrl+C        Copy selection",
            "  Shift+Arrows  Extend selection",
            "",
            "Commands",
            "  Ctrl+S        Save",
            "  Ctrl+Q        Quit safely",
            "  Ctrl+X        Exit immediately",
            "  Ctrl+O / N    Open file / new tab",
            "  Ctrl+Tab      Next tab",
            "  F3            Rename current file",
            "  Mouse click   Place cursor",
            "  Mouse wheel   Scroll document",
            "  Mouse drag    Select text",
            "  Ctrl+F        Find text",
            "  Ctrl+G        Go to line",
            "  Ctrl+W        Toggle soft wrap",
            "  Ctrl+L        Toggle line numbers",
            "  Ctrl+T / F2   Change theme",
            "  F1 / ?        This help",
            "",
            "Press any key to close",
        ];
        let box_w = cmp::min(w.saturating_sub(4), 54);
        let x = (w - box_w) / 2;
        let start = h.saturating_sub(lines.len() as u16) / 2;
        for (i, line) in lines.iter().enumerate() {
            if start + i as u16 >= h {
                break;
            }
            queue!(
                out,
                MoveTo(x, start + i as u16),
                SetBackgroundColor(self.theme.status_bg),
                SetForegroundColor(if i == 0 {
                    self.theme.accent
                } else {
                    self.theme.foreground
                }),
                SetAttribute(if i == 0 {
                    Attribute::Bold
                } else {
                    Attribute::Reset
                }),
                Print(fit(line, box_w as usize))
            )?
        }
        Ok(())
    }
}

fn char_to_byte(s: &str, n: usize) -> usize {
    s.char_indices().nth(n).map(|(i, _)| i).unwrap_or(s.len())
}
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let value = (chunk[0] as u32) << 16
            | (chunk.get(1).copied().unwrap_or(0) as u32) << 8
            | chunk.get(2).copied().unwrap_or(0) as u32;
        out.push(ALPHABET[((value >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((value >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[((value >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[(value & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}
fn cell_to_char(s: &str, target: usize) -> usize {
    let mut cells = 0;
    for (index, character) in s.chars().enumerate() {
        let width = character.width().unwrap_or(0);
        if cells + width > target {
            return index;
        }
        cells += width;
    }
    s.chars().count()
}
fn wrap_starts(s: &str, width: usize) -> Vec<usize> {
    if s.is_empty() {
        return vec![0];
    }
    let mut starts = vec![0];
    let mut cells = 0;
    for (index, character) in s.chars().enumerate() {
        let char_width = character.width().unwrap_or(0);
        if cells > 0 && cells + char_width > width {
            starts.push(index);
            cells = 0;
        }
        cells += char_width;
    }
    starts
}
fn tab_label(tab: &App, active: bool) -> String {
    let name = tab.path.file_name().unwrap_or_default().to_string_lossy();
    let mark = if tab.dirty { " •" } else { "" };
    if active {
        format!("  [{name}{mark}] ")
    } else {
        format!("  {name}{mark}  ")
    }
}
fn fit(s: &str, width: usize) -> String {
    let mut v: String = s.chars().take(width).collect();
    let n = v.chars().count();
    if n < width {
        v.push_str(&" ".repeat(width - n))
    }
    v
}
fn bar(
    out: &mut impl Write,
    y: u16,
    w: u16,
    left: &str,
    right: &str,
    bg: Color,
    fg: Color,
) -> io::Result<()> {
    let width = w as usize;
    let gap = width.saturating_sub(left.chars().count() + right.chars().count());
    let text = if gap > 0 {
        format!("{left}{}{right}", " ".repeat(gap))
    } else {
        fit(left, width)
    };
    queue!(
        out,
        MoveTo(0, y),
        SetBackgroundColor(bg),
        SetForegroundColor(fg),
        SetAttribute(Attribute::Bold),
        Print(fit(&text, width)),
        SetAttribute(Attribute::Reset)
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_terminal_cells_to_character_positions() {
        assert_eq!(cell_to_char("a界b", 0), 0);
        assert_eq!(cell_to_char("a界b", 1), 1);
        assert_eq!(cell_to_char("a界b", 2), 1);
        assert_eq!(cell_to_char("a界b", 3), 2);
        assert_eq!(cell_to_char("a界b", 4), 3);
    }

    #[test]
    fn wraps_without_splitting_wide_characters() {
        assert_eq!(wrap_starts("ab界cd", 4), vec![0, 3]);
        assert_eq!(wrap_starts("", 4), vec![0]);
    }
}
