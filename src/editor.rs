use std::{borrow::Cow, fmt::Display, io, ops::Add};

use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use ratatui_textarea::{CursorMove, DataCursor, Input, Key, TextArea};

use crate::{
    github::utils::pull_request::{Change, Changes},
    i18n::fl,
};

/// Mark the start and end of a key name in a translated message, which [`key_hints`] shows in bold
const KEY_START: char = '\u{E000}';
const KEY_END: char = '\u{E001}';

/// Marks a key name, such as `^Q`, so that [`key_hints`] shows it in bold
fn key(name: &str) -> String {
    format!("{KEY_START}{name}{KEY_END}")
}

/// Splits a translated message into spans, showing the key names marked by [`key`] in bold
fn key_hints(message: &str) -> Line<'static> {
    let bold = Style::default().add_modifier(Modifier::BOLD);
    message
        .split([KEY_START, KEY_END])
        .enumerate()
        .filter(|(_, part)| !part.is_empty())
        .map(|(index, part)| {
            // Splitting on both markers puts each key name at an odd index
            if index % 2 == 1 {
                Span::styled(part.to_owned(), bold)
            } else {
                Span::raw(part.to_owned())
            }
        })
        .collect()
}

struct SearchBox<'a> {
    textarea: TextArea<'a>,
    open: bool,
}

impl Default for SearchBox<'_> {
    fn default() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(fl!("editor-search")),
        );
        Self {
            textarea,
            open: false,
        }
    }
}

impl SearchBox<'_> {
    const fn open(&mut self) {
        self.open = true;
    }

    fn close(&mut self) {
        self.open = false;
        self.textarea.move_cursor(CursorMove::End);
        self.textarea.delete_line_by_head();
    }

    const fn height(&self) -> u16 {
        if self.open { 3 } else { 0 }
    }

    fn input(&mut self, input: Input) -> Option<&'_ str> {
        match input {
            Input {
                key: Key::Enter, ..
            }
            | Input {
                key: Key::Char('m'),
                ctrl: true,
                ..
            } => None,
            input => {
                let modified = self.textarea.input(input);
                modified.then(|| self.textarea.lines()[0].as_str())
            }
        }
    }

    fn set_error(&mut self, err: Option<impl Display>) {
        let b = err.map_or_else(
            || {
                Block::default()
                    .borders(Borders::ALL)
                    .title(fl!("editor-search"))
            },
            |err| {
                Block::default()
                    .borders(Borders::ALL)
                    .title(fl!("editor-search-error", error = err.to_string()))
                    .style(Style::default().fg(Color::Red))
            },
        );
        self.textarea.set_block(b);
    }
}

struct Buffer<'a> {
    textarea: TextArea<'a>,
    content: &'a mut String,
    path: &'a String,
    modified: bool,
}

impl<'a> Buffer<'a> {
    fn new(path: &'a String, content: &'a mut String) -> Self {
        let mut textarea = content.lines().collect::<TextArea>();
        textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
        Self {
            textarea,
            content,
            path,
            modified: false,
        }
    }

    fn save(&mut self) -> bool {
        if !self.modified {
            return false;
        }
        *self.content = self.textarea.lines().join("\r\n").add("\r\n");
        self.modified = false;
        true
    }
}

pub struct Editor<'a> {
    current: usize,
    buffers: Vec<Buffer<'a>>,
    terminal: DefaultTerminal,
    message: Option<Cow<'static, str>>,
    search: SearchBox<'a>,
}

impl<'a> Editor<'a> {
    pub fn new(content: &'a mut Changes) -> Self {
        let buffers = content
            .iter_mut()
            .map(|Change { path, manifest }| Buffer::new(path, manifest))
            .collect::<Vec<_>>();
        let terminal = ratatui::init();
        Self {
            current: 0,
            buffers,
            terminal,
            message: None,
            search: SearchBox::default(),
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            let search_height = self.search.height();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(search_height),
                        Constraint::Min(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                );

            self.terminal.draw(|f| {
                let chunks = layout.split(f.area());

                if search_height > 0 {
                    f.render_widget(&self.search.textarea, chunks[0]);
                }

                let buffer = &self.buffers[self.current];
                let textarea = &buffer.textarea;
                f.render_widget(textarea, chunks[1]);

                // Render status line
                let modified = if buffer.modified {
                    format!(" {}", fl!("editor-modified"))
                } else {
                    String::new()
                };
                let slot = format!("[{}/{}]", self.current + 1, self.buffers.len());
                let path = format!(" {}{} ", buffer.path, modified);
                let DataCursor(row, col) = textarea.cursor();
                let cursor = format!("({},{})", row + 1, col + 1);
                let status_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Length(u16::try_from(slot.len()).unwrap_or_default()),
                            Constraint::Min(1),
                            Constraint::Length(u16::try_from(cursor.len()).unwrap_or_default()),
                        ]
                        .as_ref(),
                    )
                    .split(chunks[2]);
                let status_style = Style::default().add_modifier(Modifier::REVERSED);
                f.render_widget(Paragraph::new(slot).style(status_style), status_chunks[0]);
                f.render_widget(Paragraph::new(path).style(status_style), status_chunks[1]);
                f.render_widget(Paragraph::new(cursor).style(status_style), status_chunks[2]);

                // Render message at bottom
                let message = self.message.take().map_or_else(
                    || {
                        if search_height > 0 {
                            key_hints(&fl!(
                                "editor-search-help",
                                enter = key("Enter"),
                                close = key("Esc"),
                                next = key("^G or ↓ or ^N"),
                                previous = key("M-G or ↑ or ^P")
                            ))
                        } else {
                            key_hints(&fl!(
                                "editor-help",
                                quit = key("^Q"),
                                save = key("^S"),
                                search = key("^G"),
                                switch = key("^T")
                            ))
                        }
                    },
                    |message| Line::from(Span::raw(message)),
                );
                f.render_widget(Paragraph::new(message), chunks[3]);
            })?;

            if search_height > 0 {
                let textarea = &mut self.buffers[self.current].textarea;
                match ratatui::crossterm::event::read()?.into() {
                    Input {
                        key: Key::Char('g' | 'n'),
                        ctrl: true,
                        alt: false,
                        ..
                    }
                    | Input { key: Key::Down, .. } => {
                        if !textarea.search_forward(false) {
                            self.search.set_error(Some(fl!("editor-pattern-not-found")));
                        }
                    }
                    Input {
                        key: Key::Char('g'),
                        ctrl: false,
                        alt: true,
                        ..
                    }
                    | Input {
                        key: Key::Char('p'),
                        ctrl: true,
                        alt: false,
                        ..
                    }
                    | Input { key: Key::Up, .. } => {
                        if !textarea.search_back(false) {
                            self.search.set_error(Some(fl!("editor-pattern-not-found")));
                        }
                    }
                    Input {
                        key: Key::Enter, ..
                    } => {
                        if !textarea.search_forward(true) {
                            self.message = Some(fl!("editor-pattern-not-found").into());
                        }
                        self.search.close();
                        textarea.set_search_pattern("").unwrap();
                    }
                    Input { key: Key::Esc, .. } => {
                        self.search.close();
                        textarea.set_search_pattern("").unwrap();
                    }
                    input => {
                        if let Some(query) = self.search.input(input) {
                            let maybe_err = textarea.set_search_pattern(query).err();
                            self.search.set_error(maybe_err);
                        }
                    }
                }
            } else {
                match ratatui::crossterm::event::read()?.into() {
                    Input {
                        key: Key::Char('q'),
                        ctrl: true,
                        ..
                    } => break,
                    Input {
                        key: Key::Char('t'),
                        ctrl: true,
                        ..
                    } => {
                        self.current = (self.current + 1) % self.buffers.len();
                        let number = self.current + 1;
                        self.message = Some(fl!("editor-switched-buffer", number = number).into());
                    }
                    Input {
                        key: Key::Char('s'),
                        ctrl: true,
                        ..
                    } => {
                        self.buffers[self.current].save();
                        self.message = Some(fl!("editor-saved").into());
                    }
                    Input {
                        key: Key::Char('g'),
                        ctrl: true,
                        ..
                    } => {
                        self.search.open();
                    }
                    input => {
                        let buffer = &mut self.buffers[self.current];
                        buffer.modified =
                            buffer.content.lines().enumerate().any(|(index, line)| {
                                if let Some(textarea_line) = buffer.textarea.lines().get(index) {
                                    textarea_line != line
                                } else {
                                    true
                                }
                            });
                        buffer.textarea.input(input);
                    }
                }
            }
        }

        Ok(())
    }
}

impl Drop for Editor<'_> {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{style::Modifier, text::Line};

    use super::{key, key_hints};
    use crate::i18n::fl;

    /// Returns each span's text and whether it's bold
    fn spans(line: &Line) -> Vec<(String, bool)> {
        line.spans
            .iter()
            .map(|span| {
                (
                    span.content.to_string(),
                    span.style.add_modifier.contains(Modifier::BOLD),
                )
            })
            .collect()
    }

    #[test]
    fn key_names_are_bold() {
        let line = key_hints(&fl!(
            "editor-help",
            quit = key("^Q"),
            save = key("^S"),
            search = key("^G"),
            switch = key("^T")
        ));

        assert_eq!(
            line.to_string(),
            "Press ^Q to quit, ^S to save, ^G to search, ^T to switch buffer"
        );
        assert_eq!(
            spans(&line),
            [
                ("Press ".to_owned(), false),
                ("^Q".to_owned(), true),
                (" to quit, ".to_owned(), false),
                ("^S".to_owned(), true),
                (" to save, ".to_owned(), false),
                ("^G".to_owned(), true),
                (" to search, ".to_owned(), false),
                ("^T".to_owned(), true),
                (" to switch buffer".to_owned(), false),
            ]
        );
    }

    #[test]
    fn message_can_start_with_a_key() {
        let line = key_hints(&format!("{} quits", key("^Q")));

        assert_eq!(
            spans(&line),
            [("^Q".to_owned(), true), (" quits".to_owned(), false)]
        );
    }
}
