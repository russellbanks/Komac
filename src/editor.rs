use std::borrow::Cow;
use std::fmt::Display;
use std::io;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
use tui_textarea::{CursorMove, Input, Key, TextArea};

struct SearchBox<'a> {
    textarea: TextArea<'a>,
    open: bool,
}

impl<'a> Default for SearchBox<'a> {
    fn default() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(Block::default().borders(Borders::ALL).title("Search"));
        Self {
            textarea,
            open: false,
        }
    }
}

impl<'a> SearchBox<'a> {
    fn open(&mut self) {
        self.open = true;
    }

    fn close(&mut self) {
        self.open = false;
        self.textarea.move_cursor(CursorMove::End);
        self.textarea.delete_line_by_head();
    }

    const fn height(&self) -> u16 {
        if self.open {
            3
        } else {
            0
        }
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
            || Block::default().borders(Borders::ALL).title("Search"),
            |err| {
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Search: {err}"))
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
        *self.content = self.textarea.lines().join("\r\n");
        self.modified = false;
        true
    }
}

pub struct Editor<'a> {
    current: usize,
    buffers: Vec<Buffer<'a>>,
    term: Terminal<CrosstermBackend<io::Stdout>>,
    message: Option<Cow<'static, str>>,
    search: SearchBox<'a>,
}

impl<'a> Editor<'a> {
    pub fn new(content: &'a mut [(String, String)]) -> io::Result<Self> {
        let buffers = content
            .iter_mut()
            .map(|(path, content)| Buffer::new(path, content))
            .collect::<Vec<_>>();
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let term = Terminal::new(backend)?;
        Ok(Self {
            current: 0,
            buffers,
            term,
            message: None,
            search: SearchBox::default(),
        })
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

            self.term.draw(|f| {
                let chunks = layout.split(f.size());

                if search_height > 0 {
                    f.render_widget(self.search.textarea.widget(), chunks[0]);
                }

                let buffer = &self.buffers[self.current];
                let textarea = &buffer.textarea;
                let widget = textarea.widget();
                f.render_widget(widget, chunks[1]);

                // Render status line
                let modified = if buffer.modified { " [modified]" } else { "" };
                let slot = format!("[{}/{}]", self.current + 1, self.buffers.len());
                let path = format!(" {}{} ", buffer.path, modified);
                let (row, col) = textarea.cursor();
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
                            Line::from(vec![
                                Span::raw("Press "),
                                Span::styled(
                                    "Enter",
                                    Style::default().add_modifier(Modifier::BOLD),
                                ),
                                Span::raw(" to jump to first match and close, "),
                                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                                Span::raw(" to close, "),
                                Span::styled(
                                    "^G or ↓ or ^N",
                                    Style::default().add_modifier(Modifier::BOLD),
                                ),
                                Span::raw(" to search next, "),
                                Span::styled(
                                    "M-G or ↑ or ^P",
                                    Style::default().add_modifier(Modifier::BOLD),
                                ),
                                Span::raw(" to search previous"),
                            ])
                        } else {
                            Line::from(vec![
                                Span::raw("Press "),
                                Span::styled("^Q", Style::default().add_modifier(Modifier::BOLD)),
                                Span::raw(" to quit, "),
                                Span::styled("^S", Style::default().add_modifier(Modifier::BOLD)),
                                Span::raw(" to save, "),
                                Span::styled("^G", Style::default().add_modifier(Modifier::BOLD)),
                                Span::raw(" to search, "),
                                Span::styled("^T", Style::default().add_modifier(Modifier::BOLD)),
                                Span::raw(" to switch buffer"),
                            ])
                        }
                    },
                    |message| Line::from(Span::raw(message)),
                );
                f.render_widget(Paragraph::new(message), chunks[3]);
            })?;

            if search_height > 0 {
                let textarea = &mut self.buffers[self.current].textarea;
                match crossterm::event::read()?.into() {
                    Input {
                        key: Key::Char('g' | 'n'),
                        ctrl: true,
                        alt: false,
                        ..
                    }
                    | Input { key: Key::Down, .. } => {
                        if !textarea.search_forward(false) {
                            self.search.set_error(Some("Pattern not found"));
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
                            self.search.set_error(Some("Pattern not found"));
                        }
                    }
                    Input {
                        key: Key::Enter, ..
                    } => {
                        if !textarea.search_forward(true) {
                            self.message = Some("Pattern not found".into());
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
                match crossterm::event::read()?.into() {
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
                        self.message =
                            Some(format!("Switched to buffer #{}", self.current + 1).into());
                    }
                    Input {
                        key: Key::Char('s'),
                        ctrl: true,
                        ..
                    } => {
                        self.buffers[self.current].save();
                        self.message = Some("Saved!".into());
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
                                    if textarea_line != line {
                                        return true;
                                    }
                                    false
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

impl<'a> Drop for Editor<'a> {
    fn drop(&mut self) {
        self.term.show_cursor().unwrap();
        disable_raw_mode().unwrap();
        crossterm::execute!(
            self.term.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .unwrap();
    }
}
