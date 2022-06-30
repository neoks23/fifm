use std::{io};
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
use tui::widgets::ListState;

///Define custom stateful list, containing fields:
///state: The state to get the current state of the list, for in this case manipulating cursor position
///items: Containing the current directory items

struct StatefulList<String> {
    state: ListState,
    items: Vec<String>,
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

struct App{
    view_items: StatefulList<String>,
    items: Vec<String>,
    title: String
}

impl Default for App {
    fn default() -> App {
        let view_items = list_current_dir_with_rights();
        let cd_items = list_current_dir();
        let title = get_current_dir();
        App {
            view_items: StatefulList::with_items(view_items),
            items: cd_items,
            title
        }
    }
}

///outputs current dir for cd_items
fn list_current_dir() -> Vec<String>{
    //cmd
    let output = Command::new("ls")
        .arg("-a")
        .output()
        .expect("ls cmd failed to start");

    //convert cmd output from u8 to a valid string
    let stdout = String::from_utf8_lossy(&output.stdout);

    //convert string to string slices and insert the output  Vec<String>
    let mut cd_items: Vec<String> = stdout.split('\n').map(String::from).collect();

    //Remove unnecessary "." directory
    cd_items.remove(0);

    //Remove unnecessary whitespace index
    cd_items.pop();
    cd_items
}
///outputs current dir for view_items
fn list_current_dir_with_rights() -> Vec<String>{
    //cmd
    let output = Command::new("ls")
        .arg("-a")
        .arg("-l")
        .output()
        .expect("ls cmd failed to start");

    //convert cmd output from u8 to a valid string
    let stdout = String::from_utf8_lossy(&output.stdout);

    //convert string to string slices and insert the output  Vec<String>
    let mut cd_items: Vec<String> = stdout.split('\n').map(String::from).collect();

    //Remove the "Total" line
    cd_items.remove(0);

    //Remove unnecessary "." directory
    cd_items.remove(0);

    //Remove unnecessary whitespace index
    cd_items.pop();
    cd_items
}
///gets current directory, this is used for the title of the main block
fn get_current_dir() -> String{
    let output = Command::new("pwd")
        .output()
        .expect("ls cmd failed to start");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let cd: String = stdout.to_string();
    cd
}

///sets directory once enter is pressed and destination is a valid directory
fn set_current_dir(app: &mut App) {
    let i = match app.view_items.state.selected() {
        Some(i) => i,
        None => 0
    };

    app.title = app.items[i].to_string();
    let changed = env::set_current_dir(Path::new(&app.items[i])).is_ok();

    match changed {
        true => {
            app.title = get_current_dir();
            app.items = list_current_dir();
            app.view_items = StatefulList::with_items(list_current_dir_with_rights());
            app.view_items.state.select(Some(0));
        } ,
        _ => ()
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
        //            Constraint 1.          Constraint 2.
        .constraints([Constraint::Length(2), Constraint::Percentage(50)].as_ref())
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

    // Initialize instruction msg along with styling.
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

    //Convert separate msg and styling to one text component.
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    //render message
    f.render_widget(help_message, chunks[0]);
    //render list items
    f.render_stateful_widget(items, chunks[1], &mut app.view_items.state);
}
