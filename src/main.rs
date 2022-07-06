mod custom_io;

use std::io;
use std::error::Error;
use tui::{
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem},
    layout::{Layout, Constraint, Direction},
    Frame, Terminal
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode,enable_raw_mode,EnterAlternateScreen,LeaveAlternateScreen},
};
use tui::backend::Backend;
use tui::layout::Alignment;
use tui::widgets::{ListState, Paragraph};
use crate::custom_io::{get_current_dir, list_current_dir, set_current_dir, copy, make_command, cut, delete};

///Define custom stateful list, containing fields:
///state: The state to get the current state of the list, for in this case manipulating cursor position
///items: Containing the current directory items

struct StatefulList<String> {
    state: ListState,
    items: Vec<String>,
}

enum CommandType{
    Idle,
    Cut,
    Remove,
    Copy
}

impl<String> StatefulList<String> {

    // basic custom constructor for our StateFulList.

    fn with_items(items: Vec<String>) -> StatefulList<String> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    /// Sets the cursor to next list item.
    /// __________
    /// Suppose the cursor hits the bottom of the list
    /// In case this happens the cursor would be moved over to the first item of the list

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

    /// Sets the cursor to previous list item.
    /// __________
    /// Suppose the cursor hits the top of the list
    /// In case this happens the cursor would be moved over to the last item of the list

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

    fn unselect(&mut self) {self.state.select(None);}
}

/// App holds the state of the application.
/// __________
/// For technical reasons App holds 2 list items:
/// - items for iterating over each list value.
/// - view_items for viewing more information regarding rights over dir or file.
/// __________
/// view_items is what is displayed during each fifm session

pub struct App{
    view_items: StatefulList<String>,
    items: Vec<String>,
    command_type: CommandType,
    command: String,
    selected_item: String,
    title: String,
    man: bool
}

impl Default for App {
    fn default() -> App {
        let view_items = list_current_dir("-l".to_string());
        let cd_items = list_current_dir("-a".to_string());
        let title = get_current_dir();
        App {
            view_items: StatefulList::with_items(view_items),
            command_type: CommandType::Idle,
            command: "".to_string(),
            selected_item: "".to_string(),
            items: cd_items,
            title,
            man: true,
        }
    }
}

// entry point for fifm
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

/// run_app handles backend related stuff
fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        //call to render
        terminal.draw(|f| ui(f, &mut app))?;

        //Keypress event listener:
        //__________
        //Q -> terminates fifm session.
        //Left | Right | Escape -> unselect item.
        //Down -> goto next item in the list.
        //Up -> goto previous item in the list.
        //Enter -> Select file | goto selected directory.

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                KeyCode::Char('c') | KeyCode::Char('C') => copy(&mut app),
                KeyCode::Char('x') | KeyCode::Char('X') => cut(&mut app),
                KeyCode::Char('v') | KeyCode::Char('V') => make_command(&mut app),
                KeyCode::Char('d') | KeyCode::Char('D') => delete(&mut app),
                KeyCode::Char('m') | KeyCode::Char('M') => app.man = !app.man,
                KeyCode::Left | KeyCode::Right | KeyCode::Esc => {app.view_items.unselect(); app.title = get_current_dir()},
                KeyCode::Down => app.view_items.next(),
                KeyCode::Up => app.view_items.previous(),
                KeyCode::Enter => set_current_dir(&mut app),
                _ => {}
            }
        }
    }
}

/// ui handles frontend related stuff
fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {

    // create chunk layout
    // __________
    // Constraint 1. for instructions
    // Constraint 2. for list content block.

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(if app.man {50} else {100}), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    // insert the view_items in a Vec<ListItem> with custom styling
    let items: Vec<ListItem> =
        app.view_items.items.iter().map(|i| {
            let lines = vec![Spans::from(i.to_string())];
            ListItem::new(lines).style(Style::default().fg(Color::LightCyan))
        })
        .collect();

    // Shadow variable items for adding additional properties
    // such as creating the outer shell (block) and adding a title to this block
    // All with customizable styling
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

    let instruction_items = vec![
            Spans::from(""),
            Spans::from("FIFM - Friendly Interactive File Manager "),
            Spans::from(""),
            Spans::from("version 0.2"),
            Spans::from("Maintained by © Koen Sampers 2022"),
            Spans::from("Fifm is open source and freely distributable"),
            Spans::from(""),
            Spans::from("type m -> close manual page"),
            Spans::from("type q -> exit"),
            Spans::from("arrow keys -> move"),
            Spans::from("type enter -> enter directory"),
            Spans::from("type c -> copy"),
            Spans::from("type x -> cut"),
            Spans::from("type v -> paste"),
            Spans::from("type d -> delete"),
        ];

    let pg_items: Paragraph = Paragraph::new(instruction_items).style(Style::default().fg(Color::LightCyan)).alignment(Alignment::Center);

    let pg_items = pg_items
        .block(
            Block::default()
                .border_style(
                    Style::default().fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                )
                .borders(Borders::ALL)
                .title(
                    Span::styled("MANUAL", Style::default().fg(Color::Blue).add_modifier(Modifier::ITALIC))
                )

        );

    //render list items
    f.render_stateful_widget(items, chunks[0], &mut app.view_items.state);
    //render instructions if manual is toggled
    if app.man {f.render_widget(pg_items, chunks[1])}
}