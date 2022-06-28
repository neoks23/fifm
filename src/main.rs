use std::{io, thread, time::Duration};
use std::borrow::Cow;
use std::error::Error;
use std::io::Write;
use regex::Regex;
use tui::{
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    layout::{Layout, Constraint, Direction},
    Frame, Terminal
};
use std::process::{Command, Output};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode,enable_raw_mode,EnterAlternateScreen,LeaveAlternateScreen},
};
use tui::backend::Backend;
use tui::widgets::ListState;

use unicode_width::UnicodeWidthStr;

enum InputMode {
    Normal,
    Editing,
}

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

/// App holds the state of the application
struct App{
    /// Current value of the input box
    input: String,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,

    items: StatefulList<String>,

    title: String
}

impl Default for App {
    fn default() -> App {
        let cd_items = list_current_dir();
        let title = get_current_dir();
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            items: StatefulList::with_items(cd_items),
            title
        }
    }
}

fn list_current_dir() -> Vec<String>{
    //cmd
    let mut output = Command::new("ls")
        .arg("-a")
        .output()
        .expect("ls cmd failed to start");

    //convert items to Vec<&str>
    let stdout = String::from_utf8_lossy(&output.stdout).replace('\n', " ");
    let mut cd_items: Vec<String> = stdout.split(" ").map(String::from).collect();
    cd_items.pop();
    cd_items
}
fn get_current_dir() -> String{
    let mut output = Command::new("pwd")
        .output()
        .expect("ls cmd failed to start");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut cd: String = stdout.to_string();
    cd
}

fn main() -> Result<(), Box<dyn Error>> {
    //setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    //create app and run it
    let mut app = App::default();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('e') => app.input_mode = InputMode::Editing,
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Left | KeyCode::Right => app.items.unselect(),
                    KeyCode::Down => app.items.next(),
                    KeyCode::Up => app.items.previous(),
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        app.messages.push(app.input.drain(..).collect());
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
            }
        }
    }
}
fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let items: Vec<ListItem> =
        app.items.items.iter().map(|i| {
            let mut lines = vec![Spans::from(i.to_string())];
            ListItem::new(lines).style(Style::default().fg(Color::LightCyan))
        })
        .collect();

    let items = List::new(items)
        .block(
            Block::default()
                .border_style(
                    Style::default().fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
                )
            .borders(Borders::ALL).title(app.title.to_string())
        )
        .highlight_style(
            Style::default()
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items, chunks[0], &mut app.items.state);
}
