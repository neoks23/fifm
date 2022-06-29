use std::{io};
use std::borrow::Borrow;
use std::error::Error;
use std::env;
use std::path::Path;
use tui::{
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    layout::{Layout, Constraint, Direction},
    Frame, Terminal
};
use std::process::{Command};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode,enable_raw_mode,EnterAlternateScreen,LeaveAlternateScreen},
};
use tui::backend::Backend;
use tui::layout::Alignment;
use tui::widgets::ListState;

struct StatefulList<String> {
    state: ListState,
    items: Vec<String>,
}

impl<String> StatefulList<String> {
    fn with_items(items: Vec<String>) -> StatefulList<String> {
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
    items: StatefulList<String>,
    view_items: Vec<String>,
    title: String
}

impl Default for App {
    fn default() -> App {
        let cd_items = list_current_dir_with_rights();
        let view_items = list_current_dir();
        let title = get_current_dir();
        App {
            items: StatefulList::with_items(cd_items),
            view_items,
            title
        }
    }
}

fn list_current_dir() -> Vec<String>{
    //cmd
    let output = Command::new("ls")
        .arg("-a")
        .output()
        .expect("ls cmd failed to start");

    //convert items to Vec<&str>
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut cd_items: Vec<String> = stdout.split('\n').map(String::from).collect();
    cd_items.remove(0);
    cd_items.pop();
    cd_items
}
fn list_current_dir_with_rights() -> Vec<String>{
    //cmd
    let output = Command::new("ls")
        .arg("-a")
        .arg("-l")
        .output()
        .expect("ls cmd failed to start");

    //convert items to Vec<&str>
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut cd_items: Vec<String> = stdout.split('\n').map(String::from).collect();
    cd_items.remove(0);
    cd_items.remove(0);
    cd_items.pop();
    cd_items
}
fn get_current_dir() -> String{
    let output = Command::new("pwd")
        .output()
        .expect("ls cmd failed to start");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let cd: String = stdout.to_string();
    cd
}

fn set_current_dir(app: &mut App) {
    let i = match app.items.state.selected() {
        Some(i) => i,
        None => 0
    };

    app.title = app.view_items[i].to_string();
    let changed = env::set_current_dir(Path::new(&app.view_items[i])).is_ok();

    match changed {
        true => {
            app.title = get_current_dir();
            app.view_items = list_current_dir();
            app.items = StatefulList::with_items(list_current_dir_with_rights());
            app.items.state.select(Some(0));
        } ,
        _ => ()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    //setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    //create app and run it
    let app = App::default();
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
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                KeyCode::Left | KeyCode::Right => app.items.unselect(),
                KeyCode::Down => app.items.next(),
                KeyCode::Up => app.items.previous(),
                KeyCode::Enter => set_current_dir(&mut app),
                _ => {}
            }
        }
    }
}
fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let items: Vec<ListItem> =
        app.items.items.iter().map(|i| {
            let lines = vec![Spans::from(i.to_string())];
            ListItem::new(lines).style(Style::default().fg(Color::LightCyan))
        })
        .collect();

    let items = List::new(items)
        .block(
            Block::default()
                .border_style(
                    Style::default().fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
                )
            .borders(Borders::ALL)
                .title(
                    Span::styled(app.title.to_string(), Style::default().fg(Color::Blue).add_modifier(Modifier::ITALIC))
                )
        )
        .highlight_style(
            Style::default()
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    let (msg, style) = {
        (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("arrow keys", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to navigate, "),
                Span::styled("enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to change directory, "),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK)
        )
    };

    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);
    f.render_stateful_widget(items, chunks[1], &mut app.items.state);
}
