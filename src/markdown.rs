//! Markdown to styled terminal lines.
//!
//! Parses markdown with [`pulldown_cmark`] and turns it into ratatui [`Line`]s,
//! word-wrapped to a given width so that indentation of lists, block quotes and
//! code blocks is kept on wrapped lines.

use pulldown_cmark::{
    Alignment, BlockQuoteKind, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd,
};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use regex::Regex;
use std::sync::LazyLock;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Narrowest text column used for wrapping, however deep the indentation
/// (unless the screen itself is narrower).
const MIN_TEXT_WIDTH: usize = 8;

/// Bullets for unordered lists, by nesting depth.
const BULLETS: [&str; 3] = ["• ", "◦ ", "▪ "];

/// Renders `markdown` into styled lines that fit in `width` columns.
///
/// Headings, emphasis, inline code, code blocks, lists (including task lists),
/// block quotes, links, images, tables and rules are styled; raw HTML is
/// reduced to its text. Control characters are stripped so that a document
/// cannot send escape sequences to the terminal.
pub fn render(markdown: &str, width: u16) -> Vec<Line<'static>> {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_GFM;
    let mut renderer = Renderer::new(usize::from(width).max(1));
    for event in Parser::new_ext(markdown, options) {
        renderer.event(event);
    }
    renderer.finish()
}

/// A container block that indents the lines inside it.
enum Block {
    /// A block quote, drawn as a bar in front of each line.
    Quote,
    /// A list item; `marker` is shown on its first line, then replaced by spaces.
    Item {
        marker: Option<String>,
        indent: usize,
    },
    /// A fenced or indented code block.
    Code,
}

/// A table being collected, rendered once all its rows are known.
struct Table {
    alignments: Vec<Alignment>,
    rows: Vec<Vec<Vec<Span<'static>>>>,
    header_rows: usize,
}

/// Event-driven state of the markdown to lines conversion.
struct Renderer {
    width: usize,
    lines: Vec<Line<'static>>,
    /// Inline content of the current paragraph-like block, not yet wrapped.
    spans: Vec<Span<'static>>,
    styles: Vec<Style>,
    blocks: Vec<Block>,
    /// Open lists; `Some(n)` is the next number of an ordered list.
    lists: Vec<Option<u64>>,
    /// Destination and index of the first text span of each open link.
    links: Vec<(String, usize)>,
    /// Text of the current code block or HTML block.
    raw: Option<String>,
    table: Option<Table>,
    /// Set when a block ended, so that the next block starts after a blank line.
    blank_pending: bool,
}

impl Renderer {
    fn new(width: usize) -> Self {
        Self {
            width,
            lines: Vec::new(),
            spans: Vec::new(),
            styles: Vec::new(),
            blocks: Vec::new(),
            lists: Vec::new(),
            links: Vec::new(),
            raw: None,
            table: None,
            blank_pending: false,
        }
    }

    fn finish(mut self) -> Vec<Line<'static>> {
        self.flush(false);
        self.lines
    }

    fn event(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => match self.raw.as_mut() {
                Some(raw) => raw.push_str(&text),
                None => self.push_text(&text, self.style()),
            },
            Event::Code(code) => self.push_text(&code, self.style().patch(code_style())),
            Event::Html(html) => {
                if let Some(raw) = self.raw.as_mut() {
                    raw.push_str(&html);
                }
            }
            Event::SoftBreak => self.push_text(" ", self.style()),
            Event::HardBreak => self.flush(true),
            Event::Rule => {
                self.start_block();
                let rule = "─".repeat(self.text_width());
                self.spans.push(Span::styled(rule, dim_style()));
                self.flush(false);
                self.blank_pending = true;
            }
            Event::TaskListMarker(checked) => {
                let (mark, style) = match checked {
                    true => ("[x] ", Style::new().fg(Color::Green)),
                    false => ("[ ] ", dim_style()),
                };
                self.spans.push(Span::styled(mark, style));
            }
            Event::FootnoteReference(name) => self.push_text(&format!("[^{name}]"), self.style()),
            Event::InlineMath(math) | Event::DisplayMath(math) => {
                self.push_text(&math, self.style().patch(code_style()))
            }
            Event::InlineHtml(html) => {
                let text = html_text(&html);
                if !text.is_empty() {
                    self.push_text(&text, dim_style().add_modifier(Modifier::ITALIC));
                }
            }
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph => self.start_block(),
            Tag::Heading { level, .. } => {
                self.start_block();
                self.styles.push(heading_style(level));
            }
            Tag::BlockQuote(kind) => {
                self.start_block();
                self.blocks.push(Block::Quote);
                if let Some(kind) = kind {
                    self.spans.push(Span::styled(
                        alert_label(kind),
                        quote_style().add_modifier(Modifier::BOLD),
                    ));
                    self.flush(false);
                }
            }
            Tag::CodeBlock(kind) => {
                self.start_block();
                self.blocks.push(Block::Code);
                if let CodeBlockKind::Fenced(lang) = kind {
                    if !lang.is_empty() {
                        self.spans.push(Span::styled(
                            clean(&lang),
                            dim_style().add_modifier(Modifier::ITALIC),
                        ));
                        self.flush(false);
                    }
                }
                self.raw = Some(String::new());
            }
            Tag::HtmlBlock => {
                // The block only starts once it is known to have any text.
                self.flush(false);
                self.raw = Some(String::new());
            }
            Tag::List(first) => {
                // A list nested in a tight item follows the item's text directly.
                if self.lists.is_empty() {
                    self.start_block();
                } else {
                    self.flush(false);
                }
                self.lists.push(first);
            }
            Tag::Item => {
                self.flush(false);
                let marker = match self.lists.last_mut() {
                    Some(Some(number)) => {
                        *number += 1;
                        format!("{}. ", *number - 1)
                    }
                    _ => BULLETS[(self.lists.len().max(1) - 1) % BULLETS.len()].to_string(),
                };
                let indent = marker.width();
                self.blocks.push(Block::Item {
                    marker: Some(marker),
                    indent,
                });
            }
            Tag::Table(alignments) => {
                self.start_block();
                self.table = Some(Table {
                    alignments,
                    rows: Vec::new(),
                    header_rows: 0,
                });
            }
            Tag::TableHead | Tag::TableRow => {
                if let Some(table) = self.table.as_mut() {
                    table.rows.push(Vec::new());
                }
                if matches!(tag, Tag::TableHead) {
                    self.styles.push(Style::new().add_modifier(Modifier::BOLD));
                }
            }
            Tag::Emphasis => self
                .styles
                .push(Style::new().add_modifier(Modifier::ITALIC)),
            Tag::Strong => self.styles.push(Style::new().add_modifier(Modifier::BOLD)),
            Tag::Strikethrough => self
                .styles
                .push(Style::new().add_modifier(Modifier::CROSSED_OUT)),
            Tag::Link { dest_url, .. } => {
                self.links.push((dest_url.to_string(), self.spans.len()));
                self.styles.push(link_style());
            }
            Tag::Image { .. } => {
                self.styles.push(dim_style().add_modifier(Modifier::ITALIC));
                self.push_text("[image: ", self.style());
            }
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => self.end_block(),
            TagEnd::Heading(level) => {
                let heading_width = self.spans.iter().map(Span::width).sum::<usize>();
                self.styles.pop();
                self.end_block();
                let underline = match level {
                    HeadingLevel::H1 => "═",
                    HeadingLevel::H2 => "─",
                    _ => return,
                };
                let rule = underline.repeat(heading_width.min(self.text_width()));
                self.spans.push(Span::styled(
                    rule,
                    heading_style(level).remove_modifier(Modifier::UNDERLINED),
                ));
                self.flush(false);
            }
            TagEnd::BlockQuote(_) => {
                self.flush(false);
                self.blocks.pop();
                self.blank_pending = true;
            }
            TagEnd::CodeBlock => {
                let code = self.raw.take().unwrap_or_default();
                for line in code.strip_suffix('\n').unwrap_or(&code).split('\n') {
                    self.spans.push(Span::styled(clean(line), code_style()));
                    self.flush(true);
                }
                self.blocks.pop();
                self.blank_pending = true;
            }
            TagEnd::HtmlBlock => {
                let html = self.raw.take().unwrap_or_default();
                let text = html_text(&html);
                let mut lines = text
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .peekable();
                if lines.peek().is_some() {
                    self.start_block();
                    for line in lines {
                        self.push_text(line, dim_style());
                        self.flush(false);
                    }
                    self.blank_pending = true;
                }
            }
            TagEnd::List(_) => {
                self.lists.pop();
                self.blank_pending = true;
            }
            TagEnd::Item => {
                let marker_unused = matches!(
                    self.blocks.last(),
                    Some(Block::Item {
                        marker: Some(_),
                        ..
                    })
                );
                self.flush(marker_unused);
                self.blocks.pop();
            }
            TagEnd::TableCell => {
                let cell = std::mem::take(&mut self.spans);
                if let Some(row) = self.table.as_mut().and_then(|table| table.rows.last_mut()) {
                    row.push(cell);
                }
            }
            TagEnd::TableHead => {
                self.styles.pop();
                if let Some(table) = self.table.as_mut() {
                    table.header_rows = table.rows.len();
                }
            }
            TagEnd::Table => {
                if let Some(table) = self.table.take() {
                    self.push_table(table);
                }
                self.blank_pending = true;
            }
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => {
                self.styles.pop();
            }
            TagEnd::Link => {
                self.styles.pop();
                if let Some((url, first)) = self.links.pop() {
                    let text: String = self.spans[first..]
                        .iter()
                        .map(|span| span.content.as_ref())
                        .collect();
                    if url.is_empty() || url.starts_with('#') || text == url {
                        // The text says it all.
                    } else if text.trim().is_empty() {
                        self.push_text(&url, link_style());
                    } else {
                        self.push_text(&format!(" <{url}>"), dim_style());
                    }
                }
            }
            TagEnd::Image => {
                self.push_text("]", self.style());
                self.styles.pop();
            }
            _ => {}
        }
    }

    /// Starts a block, separated by a blank line from the block before it.
    fn start_block(&mut self) {
        self.flush(false);
        if self.blank_pending && !self.lines.is_empty() {
            let mut blank = Line::from(self.prefix(false));
            if let Some(last) = blank.spans.last_mut() {
                last.content = last.content.trim_end().to_string().into();
            }
            self.lines.push(blank);
        }
        self.blank_pending = false;
    }

    fn end_block(&mut self) {
        self.flush(false);
        self.blank_pending = true;
    }

    fn style(&self) -> Style {
        self.styles
            .iter()
            .fold(Style::new(), |style, next| style.patch(*next))
    }

    fn push_text(&mut self, text: &str, style: Style) {
        self.spans.push(Span::styled(clean(text), style));
    }

    /// Width left for text once the current indentation is drawn.
    fn text_width(&self) -> usize {
        let indent: usize = self.prefix(false).iter().map(Span::width).sum();
        self.width
            .saturating_sub(indent)
            .max(MIN_TEXT_WIDTH.min(self.width))
    }

    /// Builds the indentation for a line from the open container blocks.
    ///
    /// With `first`, list markers not yet shown are used (and marked shown);
    /// otherwise they are replaced by spaces of the same width.
    fn prefix(&self, first: bool) -> Vec<Span<'static>> {
        self.blocks
            .iter()
            .map(|block| match block {
                Block::Quote => Span::styled("┃ ", quote_style()),
                Block::Item {
                    marker: Some(marker),
                    ..
                } if first => Span::styled(marker.clone(), Style::new().fg(Color::Cyan)),
                Block::Item { indent, .. } => Span::raw(" ".repeat(*indent)),
                Block::Code => Span::styled("  ", code_style()),
            })
            .collect()
    }

    /// The indentation for the first line of a block: shows the list markers
    /// not yet shown, and marks them shown.
    fn first_prefix(&mut self) -> Vec<Span<'static>> {
        let prefix = self.prefix(true);
        for block in &mut self.blocks {
            if let Block::Item { marker, .. } = block {
                marker.take();
            }
        }
        prefix
    }

    /// Wraps the pending inline content into lines with the current indentation.
    ///
    /// Does nothing if there is no content, unless `force` is set (empty code
    /// lines, hard breaks and empty list items still take a line).
    fn flush(&mut self, force: bool) {
        if self.table.is_some() || (self.spans.is_empty() && !force) {
            return;
        }
        let rest = self.prefix(false);
        let first = self.first_prefix();
        let spans = std::mem::take(&mut self.spans);
        // Open links (split by a hard break) now start at the new line.
        for (_, first) in &mut self.links {
            *first = 0;
        }
        let width = self.text_width();
        let wrapped = wrap(&spans, width);
        for (index, mut line) in wrapped.into_iter().enumerate() {
            let mut prefixed = if index == 0 {
                first.clone()
            } else {
                rest.clone()
            };
            prefixed.append(&mut line);
            self.lines.push(Line::from(prefixed));
        }
    }

    fn push_table(&mut self, table: Table) {
        let columns = table.rows.iter().map(Vec::len).max().unwrap_or(0);
        let widths: Vec<usize> = (0..columns)
            .map(|column| {
                table
                    .rows
                    .iter()
                    .filter_map(|row| row.get(column))
                    .map(|cell| cell.iter().map(Span::width).sum::<usize>())
                    .max()
                    .unwrap_or(0)
            })
            .collect();
        let separator = Span::styled(" │ ", dim_style());
        for (index, row) in table.rows.into_iter().enumerate() {
            let mut line = self.first_prefix();
            for (column, width) in widths.iter().enumerate() {
                if column > 0 {
                    line.push(separator.clone());
                }
                let cell = row.get(column).cloned().unwrap_or_default();
                let padding = width - cell.iter().map(Span::width).sum::<usize>();
                let (left, right) = match table.alignments.get(column) {
                    Some(Alignment::Right) => (padding, 0),
                    Some(Alignment::Center) => (padding / 2, padding - padding / 2),
                    _ => (0, padding),
                };
                line.push(Span::raw(" ".repeat(left)));
                line.extend(cell);
                line.push(Span::raw(" ".repeat(right)));
            }
            self.lines.push(Line::from(line));
            if index + 1 == table.header_rows {
                let mut rule = self.prefix(false);
                let cells: Vec<String> = widths.iter().map(|width| "─".repeat(*width)).collect();
                rule.push(Span::styled(cells.join("─┼─"), dim_style()));
                self.lines.push(Line::from(rule));
            }
        }
    }
}

/// A run of spans that is either whitespace or a word (which may mix styles).
struct Segment {
    spans: Vec<Span<'static>>,
    width: usize,
    space: bool,
}

/// Splits styled spans into words and whitespace runs.
fn segments(spans: &[Span<'static>]) -> Vec<Segment> {
    let mut segments: Vec<Segment> = Vec::new();
    for span in spans {
        let mut rest = span.content.as_ref();
        while let Some(first) = rest.chars().next() {
            let space = first.is_whitespace();
            let end = rest
                .find(|c: char| c.is_whitespace() != space)
                .unwrap_or(rest.len());
            let (piece, tail) = rest.split_at(end);
            rest = tail;
            let piece = Span::styled(piece.to_string(), span.style);
            match segments.last_mut() {
                Some(last) if last.space == space => {
                    last.width += piece.width();
                    last.spans.push(piece);
                }
                _ => segments.push(Segment {
                    width: piece.width(),
                    spans: vec![piece],
                    space,
                }),
            }
        }
    }
    segments
}

/// Greedily wraps styled spans to `width` columns.
///
/// Leading whitespace is kept on the first line (indented code) but dropped on
/// wrapped lines; words wider than a line are split. Always returns at least
/// one (possibly empty) line.
fn wrap(spans: &[Span<'static>], width: usize) -> Vec<Vec<Span<'static>>> {
    let mut wrapper = Wrapper {
        lines: vec![Vec::new()],
        used: 0,
        width,
    };
    for segment in segments(spans) {
        if segment.space {
            if wrapper.used == 0 && wrapper.lines.len() > 1 {
                continue;
            }
            if wrapper.used + segment.width > width {
                wrapper.new_line();
                continue;
            }
        } else if wrapper.used > 0 && wrapper.used + segment.width > width {
            wrapper.new_line();
        }
        for span in segment.spans {
            wrapper.push(span);
        }
    }
    wrapper.lines
}

/// Lines being filled by [`wrap`].
struct Wrapper {
    lines: Vec<Vec<Span<'static>>>,
    /// Columns used on the last line.
    used: usize,
    width: usize,
}

impl Wrapper {
    fn last(&mut self) -> &mut Vec<Span<'static>> {
        self.lines.last_mut().expect("a wrapper always has a line")
    }

    fn new_line(&mut self) {
        trim_trailing_space(self.last());
        self.lines.push(Vec::new());
        self.used = 0;
    }

    /// Appends a span, breaking it between characters where it does not fit.
    fn push(&mut self, span: Span<'static>) {
        if self.used + span.width() <= self.width {
            self.used += span.width();
            self.last().push(span);
            return;
        }
        let mut piece = String::new();
        for c in span.content.chars() {
            let c_width = c.width().unwrap_or(0);
            if self.used > 0 && self.used + c_width > self.width {
                if !piece.is_empty() {
                    let full = Span::styled(std::mem::take(&mut piece), span.style);
                    self.last().push(full);
                }
                self.new_line();
            }
            piece.push(c);
            self.used += c_width;
        }
        if !piece.is_empty() {
            self.last().push(Span::styled(piece, span.style));
        }
    }
}

fn trim_trailing_space(line: &mut Vec<Span<'static>>) {
    while let Some(last) = line.last_mut() {
        let trimmed = last.content.trim_end().to_string();
        if trimmed.is_empty() {
            line.pop();
        } else {
            last.content = trimmed.into();
            return;
        }
    }
}

/// Expands tabs and drops other control characters.
fn clean(text: &str) -> String {
    text.chars()
        .flat_map(|c| match c {
            '\t' => vec![' '; 4],
            c if c.is_control() => vec![],
            c => vec![c],
        })
        .collect()
}

/// Reduces raw HTML to its text content.
///
/// Images become `[image: <alt>]` like markdown images; other tags and comments
/// are removed.
fn html_text(html: &str) -> String {
    static IMAGE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"(?i)<img\b[^>]*?\balt\s*=\s*"([^"]*)"[^>]*>"#).expect("valid regex")
    });
    static TAG: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?s)<!--.*?-->|<[^>]*>").expect("valid regex"));
    let html = IMAGE.replace_all(html, "[image: $1]");
    TAG.replace_all(&html, "").into_owned()
}

fn alert_label(kind: BlockQuoteKind) -> &'static str {
    match kind {
        BlockQuoteKind::Note => "Note",
        BlockQuoteKind::Tip => "Tip",
        BlockQuoteKind::Important => "Important",
        BlockQuoteKind::Warning => "Warning",
        BlockQuoteKind::Caution => "Caution",
    }
}

fn heading_style(level: HeadingLevel) -> Style {
    let style = Style::new().add_modifier(Modifier::BOLD);
    match level {
        HeadingLevel::H1 => style.fg(Color::Magenta),
        HeadingLevel::H2 => style.fg(Color::Cyan),
        HeadingLevel::H3 => style.fg(Color::Blue),
        _ => style.add_modifier(Modifier::UNDERLINED),
    }
}

fn code_style() -> Style {
    Style::new().fg(Color::Yellow)
}

fn link_style() -> Style {
    Style::new()
        .fg(Color::Blue)
        .add_modifier(Modifier::UNDERLINED)
}

fn quote_style() -> Style {
    Style::new().fg(Color::Gray).add_modifier(Modifier::ITALIC)
}

fn dim_style() -> Style {
    Style::new().fg(Color::DarkGray)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plain text of each rendered line.
    fn texts(lines: &[Line<'_>]) -> Vec<String> {
        lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect()
            })
            .collect()
    }

    /// The first span whose text is exactly `content`.
    fn span<'a>(lines: &'a [Line<'static>], content: &str) -> &'a Span<'static> {
        lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .find(|span| span.content == content)
            .unwrap_or_else(|| panic!("no span {content:?}"))
    }

    #[test]
    fn test_render_headings_are_styled_and_underlined() {
        let lines = render("# Title\n\n## Sub\n\n### Third", 40);
        assert_eq!(
            texts(&lines),
            ["Title", "═════", "", "Sub", "───", "", "Third"]
        );
        assert_eq!(span(&lines, "Title").style, heading_style(HeadingLevel::H1));
        assert!(span(&lines, "Third")
            .style
            .add_modifier
            .contains(Modifier::BOLD));
    }

    #[test]
    fn test_render_inline_styles() {
        let lines = render("*it* **bold** ~~gone~~ `code`", 40);
        assert!(span(&lines, "it")
            .style
            .add_modifier
            .contains(Modifier::ITALIC));
        assert!(span(&lines, "bold")
            .style
            .add_modifier
            .contains(Modifier::BOLD));
        assert!(span(&lines, "gone")
            .style
            .add_modifier
            .contains(Modifier::CROSSED_OUT));
        assert_eq!(span(&lines, "code").style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_render_links_show_url_except_anchors() {
        let lines = render("[site](https://example.com) [top](#top) <https://a.b>", 80);
        assert_eq!(
            texts(&lines),
            ["site <https://example.com> top https://a.b"]
        );
        assert_eq!(span(&lines, "site").style, link_style());
    }

    #[test]
    fn test_render_lists_wrap_under_their_marker() {
        let lines = render(
            "- alpha beta gamma delta\n  - nested\n\n3. three\n4. four",
            16,
        );
        assert_eq!(
            texts(&lines),
            [
                "• alpha beta",
                "  gamma delta",
                "  ◦ nested",
                "",
                "3. three",
                "4. four"
            ]
        );
    }

    #[test]
    fn test_render_table_in_list_item_shows_marker_once() {
        let lines = render("- | a |\n  |---|\n  | b |", 40);
        assert_eq!(texts(&lines), ["• a", "  ─", "  b"]);
    }

    #[test]
    fn test_render_hard_break_inside_link() {
        let lines = render("*a* *b* [one\\\ntwo](https://example.com)", 40);
        assert_eq!(texts(&lines), ["a b one", "two <https://example.com>"]);
    }

    #[test]
    fn test_render_task_list() {
        let lines = render("- [x] done\n- [ ] todo", 40);
        assert_eq!(texts(&lines), ["• [x] done", "• [ ] todo"]);
    }

    #[test]
    fn test_render_block_quote_keeps_bar_on_wrapped_lines() {
        let lines = render("> one two three four", 12);
        assert_eq!(texts(&lines), ["┃ one two", "┃ three four"]);
    }

    #[test]
    fn test_render_code_block_keeps_indentation() {
        let lines = render("```sh\nif true; then\n\techo hi\nfi\n```", 40);
        assert_eq!(
            texts(&lines),
            ["  sh", "  if true; then", "      echo hi", "  fi"]
        );
        assert_eq!(span(&lines, "fi").style, code_style());
    }

    #[test]
    fn test_render_table_aligns_columns() {
        let lines = render("| a | b |\n|---|--:|\n| xyz | 1 |", 40);
        assert_eq!(texts(&lines), ["a   │ b", "────┼──", "xyz │ 1"]);
        assert!(span(&lines, "a")
            .style
            .add_modifier
            .contains(Modifier::BOLD));
    }

    #[test]
    fn test_render_html_is_reduced_to_text() {
        let lines = render("<p align=\"center\"><img src=\"x.png\">Hello</p>\n", 40);
        assert_eq!(texts(&lines), ["Hello"]);
    }

    #[test]
    fn test_render_empty_html_block_adds_no_blank_line() {
        let lines = render("one\n\n<br>\n\ntwo", 40);
        assert_eq!(texts(&lines), ["one", "", "two"]);
    }

    #[test]
    fn test_render_inline_html_image_shows_alt_text() {
        let lines = render(
            "[<img alt=\"ci\" src=\"b.svg\">](https://ci.example) ![logo](l.png)",
            80,
        );
        assert_eq!(
            texts(&lines),
            ["[image: ci] <https://ci.example> [image: logo]"]
        );
    }

    #[test]
    fn test_render_link_without_text_shows_url() {
        let lines = render("[<span></span>](https://example.com)", 80);
        assert_eq!(texts(&lines), ["https://example.com"]);
    }

    #[test]
    fn test_render_strips_control_characters() {
        let lines = render("safe\u{1b}[2J text\u{7}", 40);
        assert_eq!(texts(&lines), ["safe[2J text"]);
    }

    #[test]
    fn test_render_never_exceeds_width() {
        let markdown = "# A heading that is rather long\n\n\
            Some text with averyveryverylongwordthatcannotfitanywhere in it.\n\n\
            - item with `code spans` and **bold words** going on and on\n\n\
            > quoted text that wraps\n\n\
            ```\nlet x = \"a long line of code that must be broken somewhere\";\n```\n";
        for width in [12u16, 20, 33, 80] {
            for line in render(markdown, width) {
                assert!(
                    line.width() <= usize::from(width),
                    "{line:?} wider than {width}"
                );
            }
        }
    }

    #[test]
    fn test_wrap_splits_long_words() {
        let lines = wrap(&[Span::raw("abcdefghij")], 4);
        let texts: Vec<String> = lines
            .iter()
            .map(|line| line.iter().map(|span| span.content.as_ref()).collect())
            .collect();
        assert_eq!(texts, ["abcd", "efgh", "ij"]);
    }
}
