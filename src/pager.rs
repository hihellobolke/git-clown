//! A scrollable terminal pager for markdown documents.
//!
//! Renders markdown with [`crate::markdown`] in a full-screen ratatui view
//! driven by crossterm key events. When stdout is not a terminal (or `TERM` is
//! `dumb`) the markdown is printed as-is instead.

use crate::markdown;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use std::{
    env,
    io::{self, IsTerminal, Write},
};

/// Key help shown in the status bar.
const KEY_HELP: &str = "q quit · ↑↓ j k scroll · space b page · g G top/end";

/// Status bar hint shown when some lines are wider than the screen.
const SIDE_SCROLL_HELP: &str = "←→ h l: more to the side";

/// Columns moved by one sideways scroll step.
const SIDE_STEP: usize = 8;

/// Shows `markdown` in the pager, or prints it unchanged when stdout is not an
/// interactive terminal.
///
/// Blocks until the user quits the pager.
///
/// # Errors
///
/// Returns IO errors from setting up the terminal, reading key events or
/// writing to stdout.
pub fn show(title: &str, markdown: &str) -> io::Result<()> {
    if !stdout_is_interactive() {
        return print_plain(&mut io::stdout().lock(), markdown);
    }

    let mut terminal = ratatui::try_init()?;
    let result = run(&mut terminal, Pager::new(title, markdown));
    ratatui::try_restore()?;
    result
}

/// Writes the markdown source unchanged, ending with a newline.
fn print_plain(out: &mut impl Write, markdown: &str) -> io::Result<()> {
    out.write_all(markdown.as_bytes())?;
    if !markdown.ends_with('\n') {
        out.write_all(b"\n")?;
    }
    out.flush()
}

/// Whether stdout is a terminal that can show the full-screen pager.
fn stdout_is_interactive() -> bool {
    io::stdout().is_terminal() && env::var("TERM").map_or(true, |term| term != "dumb")
}

/// Draws the pager and handles key events until the user quits.
fn run(terminal: &mut ratatui::DefaultTerminal, mut pager: Pager) -> io::Result<()> {
    loop {
        terminal.draw(|frame| pager.draw(frame))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && !pager.handle_key(key) {
                return Ok(());
            }
        }
    }
}

/// State of the pager: the document, its lines at the current width and the
/// scroll position.
struct Pager {
    title: String,
    markdown: String,
    lines: Vec<Line<'static>>,
    /// Width `lines` were wrapped to; `None` before the first draw.
    width: Option<u16>,
    /// Number of document lines that fit on screen.
    height: usize,
    /// Index of the first line shown.
    offset: usize,
    /// Columns the widest line overflows the screen by.
    overflow: usize,
    /// Index of the first column shown.
    column: usize,
}

impl Pager {
    fn new(title: &str, markdown: &str) -> Self {
        Self {
            title: title.to_string(),
            markdown: markdown.to_string(),
            lines: Vec::new(),
            width: None,
            height: 0,
            offset: 0,
            overflow: 0,
            column: 0,
        }
    }

    /// Re-renders the document if the screen size changed.
    fn resize(&mut self, width: u16, height: u16) {
        if self.width != Some(width) {
            self.lines = markdown::render(&self.markdown, width);
            self.width = Some(width);
            let widest = self.lines.iter().map(Line::width).max().unwrap_or(0);
            self.overflow = widest.saturating_sub(usize::from(width));
        }
        self.height = usize::from(height);
        self.offset = self.offset.min(self.max_offset());
        self.column = self.column.min(self.overflow);
    }

    fn max_offset(&self) -> usize {
        self.lines.len().saturating_sub(self.height)
    }

    fn scroll_down(&mut self, lines: usize) {
        self.offset = (self.offset + lines).min(self.max_offset());
    }

    fn scroll_up(&mut self, lines: usize) {
        self.offset = self.offset.saturating_sub(lines);
    }

    /// Applies a key press. Returns `false` when the pager should close.
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        let page = self.height.max(1);
        // Ctrl-F/B/D/U behave like their plain letters, as in less and vim.
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return false,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return false,
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Enter => self.scroll_down(1),
            KeyCode::Up | KeyCode::Char('k') => self.scroll_up(1),
            KeyCode::PageDown | KeyCode::Char(' ') | KeyCode::Char('f') => self.scroll_down(page),
            KeyCode::PageUp | KeyCode::Char('b') => self.scroll_up(page),
            KeyCode::Char('d') => self.scroll_down(page / 2),
            KeyCode::Char('u') => self.scroll_up(page / 2),
            KeyCode::Home | KeyCode::Char('g') => self.offset = 0,
            KeyCode::End | KeyCode::Char('G') => self.offset = self.max_offset(),
            KeyCode::Left | KeyCode::Char('h') => {
                self.column = self.column.saturating_sub(SIDE_STEP)
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.column = (self.column + SIDE_STEP).min(self.overflow)
            }
            _ => {}
        }
        true
    }

    /// Draws the visible lines and a status bar on the last screen row.
    fn draw(&mut self, frame: &mut Frame) {
        let [body, status] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());
        self.resize(body.width, body.height);

        let end = (self.offset + self.height).min(self.lines.len());
        let visible = self.lines[self.offset..end].to_vec();
        let column = u16::try_from(self.column).unwrap_or(u16::MAX);
        frame.render_widget(Paragraph::new(visible).scroll((0, column)), body);
        frame.render_widget(Paragraph::new(self.status_line(end)), status);
    }

    fn status_line(&self, end: usize) -> Line<'static> {
        let total = self.lines.len();
        let percent = if total == 0 { 100 } else { end * 100 / total };
        let position = format!(
            " {}-{}/{} ({percent}%) ",
            (self.offset + 1).min(total),
            end,
            total
        );
        let bar = Style::new().add_modifier(Modifier::REVERSED);
        let mut spans = vec![
            Span::styled(
                format!(" {} ", self.title),
                bar.add_modifier(Modifier::BOLD),
            ),
            Span::styled(position, bar),
        ];
        // Placed before the key help so that it is not cut off on narrow screens.
        if self.overflow > 0 {
            spans.push(Span::styled(
                format!(" {SIDE_SCROLL_HELP} "),
                bar.add_modifier(Modifier::BOLD),
            ));
        }
        spans.push(Span::styled(format!(" {KEY_HELP} "), bar));
        Line::from(spans).style(bar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    /// A pager over `count` one-line paragraphs, sized to 40x`height`.
    fn pager(count: usize, height: u16) -> Pager {
        let markdown: Vec<String> = (1..=count).map(|n| format!("line {n}\n")).collect();
        let mut pager = Pager::new("doc", &markdown.join("\n"));
        pager.resize(40, height);
        pager
    }

    fn press(pager: &mut Pager, code: KeyCode) -> bool {
        pager.handle_key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    #[test]
    fn test_print_plain_keeps_source_and_ends_with_newline() {
        let mut out = Vec::new();
        print_plain(&mut out, "# Title\n\n*text*").unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "# Title\n\n*text*\n");
    }

    #[test]
    fn test_pager_scrolls_within_bounds() {
        // 10 paragraphs render to 19 lines (blank lines between them).
        let mut pager = pager(10, 5);
        assert_eq!(pager.max_offset(), 14);

        press(&mut pager, KeyCode::Up);
        assert_eq!(pager.offset, 0);
        press(&mut pager, KeyCode::Char('j'));
        assert_eq!(pager.offset, 1);
        press(&mut pager, KeyCode::Char(' '));
        assert_eq!(pager.offset, 6);
        press(&mut pager, KeyCode::Char('G'));
        assert_eq!(pager.offset, 14);
        press(&mut pager, KeyCode::PageDown);
        assert_eq!(pager.offset, 14);
        press(&mut pager, KeyCode::Char('u'));
        assert_eq!(pager.offset, 12);
        press(&mut pager, KeyCode::Char('g'));
        assert_eq!(pager.offset, 0);
    }

    #[test]
    fn test_pager_quit_keys() {
        let mut pager = pager(3, 5);
        assert!(press(&mut pager, KeyCode::Char('j')));
        assert!(!press(&mut pager, KeyCode::Char('q')));
        assert!(!press(&mut pager, KeyCode::Esc));
        assert!(!pager.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)));
    }

    #[test]
    fn test_pager_resize_rewraps_and_clamps_offset() {
        let mut pager = pager(10, 5);
        press(&mut pager, KeyCode::End);
        pager.resize(40, 30);
        assert_eq!(pager.offset, 0);
        pager.resize(4, 30);
        assert!(pager.lines.len() > 19, "narrow width should wrap lines");
    }

    #[test]
    fn test_pager_scrolls_sideways_only_when_lines_overflow() {
        let mut pager = pager(3, 5);
        assert_eq!(pager.overflow, 0);
        press(&mut pager, KeyCode::Right);
        assert_eq!(pager.column, 0);

        // Text wraps, but a table with more columns than fit cannot.
        let wide = format!("|{}\n|{}", "a|".repeat(15), "-|".repeat(15));
        let mut pager = Pager::new("doc", &wide);
        pager.resize(20, 5);
        assert!(pager.overflow > 0);
        press(&mut pager, KeyCode::Char('l'));
        assert_eq!(pager.column, SIDE_STEP.min(pager.overflow));
        for _ in 0..20 {
            press(&mut pager, KeyCode::Right);
        }
        assert_eq!(pager.column, pager.overflow);
        press(&mut pager, KeyCode::Left);
        assert_eq!(pager.column, pager.overflow - SIDE_STEP.min(pager.overflow));
        for _ in 0..20 {
            press(&mut pager, KeyCode::Char('h'));
        }
        assert_eq!(pager.column, 0);

        // Widening the screen removes the overflow and resets the column.
        press(&mut pager, KeyCode::End);
        press(&mut pager, KeyCode::Right);
        pager.resize(200, 5);
        assert_eq!((pager.overflow, pager.column), (0, 0));
    }

    #[test]
    fn test_pager_draws_overflowing_lines_scrolled_sideways() {
        let wide = format!("|{}\n|{}", "ab|".repeat(15), "-|".repeat(15));
        let mut terminal = Terminal::new(TestBackend::new(30, 4)).unwrap();
        let mut pager = Pager::new("doc", &wide);
        terminal.draw(|frame| pager.draw(frame)).unwrap();
        let row = |terminal: &Terminal<TestBackend>, y: u16| -> String {
            let buffer = terminal.backend().buffer();
            (0..30).map(|x| buffer[(x, y)].symbol()).collect()
        };
        // Squeezed to one column each, the cells wrap onto two lines.
        assert!(row(&terminal, 0).starts_with("a │ a │ a"));
        assert!(row(&terminal, 1).starts_with("b │ b │ b"));
        assert!(
            row(&terminal, 3).contains("←→ h l"),
            "{}",
            row(&terminal, 3)
        );

        pager.handle_key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
        for _ in 0..20 {
            pager.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        }
        terminal.draw(|frame| pager.draw(frame)).unwrap();
        assert!(
            row(&terminal, 0).trim_end().ends_with("a │ a"),
            "{}",
            row(&terminal, 0)
        );
    }

    #[test]
    fn test_pager_draws_document_and_status_bar() {
        let mut terminal = Terminal::new(TestBackend::new(80, 4)).unwrap();
        let mut pager = Pager::new("README.md", "# Hello\n\nworld");
        terminal.draw(|frame| pager.draw(frame)).unwrap();

        let buffer = terminal.backend().buffer();
        let row = |y: u16| -> String { (0..80).map(|x| buffer[(x, y)].symbol()).collect() };
        assert!(row(0).starts_with("Hello"));
        assert!(row(1).starts_with("═════"));
        assert!(row(3).starts_with(" README.md  1-3/4 (75%)"), "{}", row(3));
    }
}
