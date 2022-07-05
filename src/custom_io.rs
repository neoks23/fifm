use std::env;
use fs_extra::error::Error;
use std::path::Path;
use std::process::Command;
use crate::{App, CommandType, StatefulList};


pub fn list_current_dir_matches(grep: String) -> usize {
    let output = Command::new("ls")
        .arg("-a")
        .arg(format!("| grep {}", grep))
        .output()
        .expect("ls cmd failed to start");

    let stdout = String::from_utf8_lossy(&output.stdout);

    //convert string to string slices and insert the output  Vec<String>
    let mut cd_items: Vec<String> = stdout.split('\n').map(String::from).collect();
    cd_items.len()
}
///outputs current dir for view_items
pub fn list_current_dir(arg: String) -> Vec<String>{
    //cmd
    let output = Command::new("ls")
        .arg("-a")
        .arg(arg.as_str())
        .output()
        .expect("ls cmd failed to start");

    //convert cmd output from u8 to a valid string
    let stdout = String::from_utf8_lossy(&output.stdout);

    //convert string to string slices and insert the output  Vec<String>
    let mut cd_items: Vec<String> = stdout.split('\n').map(String::from).collect();

    //Remove the "Total" line if it exists
    if arg.as_str() == "-l" {
        cd_items.remove(0);
    }

    //Remove unnecessary "." directory
    cd_items.remove(0);

    //Remove unnecessary whitespace index
    cd_items.pop();
    cd_items
}
///gets current directory, this is used for the title of the main block
pub fn get_current_dir() -> String{
    let output = Command::new("pwd")
        .output()
        .expect("ls cmd failed to start");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let cd: String = stdout.to_string();
    cd
}

///sets directory once enter is pressed and destination is a valid directory
pub fn set_current_dir(app: &mut App) {
    let i = match app.view_items.state.selected() {
        Some(i) => i,
        None => 0
    };

    app.title = app.items[i].to_string();
    let changed = env::set_current_dir(Path::new(&app.items[i])).is_ok();

    match changed {
        true => {
            app.title = get_current_dir();
            app.items = list_current_dir("-a".to_string());
            app.view_items = StatefulList::with_items(list_current_dir("-l".to_string()));
            app.view_items.state.select(Some(0));
        } ,
        _ => ()
    }
}
fn select(app: &mut App, cmd_type: CommandType) {
    app.command_type = cmd_type;

    let i = match app.view_items.state.selected() {
        Some(i) => i,
        None => 0
    };
    app.selected_item = app.items[i].to_string();
}

pub fn copy(app: &mut App){
    select(app, CommandType::Copy);
    let mut cd = get_current_dir();
    cd = cd.trim().parse().unwrap();
    cd.push_str("/");
    cd.push_str( app.selected_item.as_str());
    app.command = cd;
    app.title = format!("Copied {}",  app.command.as_str());
}
pub fn move_file(app: &mut App){
    copy(app);
    app.title = format!("Cut {}",  app.command.as_str());
    select(app, CommandType::Move);
}
pub fn delete(app: &mut App) {
    select(app, CommandType::Remove);
    make_command(app);
}

pub fn make_command(app: &mut App){
    match app.command_type {
        CommandType::Copy => {

            let mut cd =  get_current_dir();
            cd = cd.trim().parse().unwrap();
            cd.push_str("/");
            cd.push_str(app.selected_item.as_str());

            let md = Path::new(app.command.as_str());

            if app.command.to_string() == cd && md.is_file() {
                cd.clear();
                cd =  get_current_dir();
                cd = cd.trim().parse().unwrap();
                cd.push_str("/");
                let size = list_current_dir_matches(app.selected_item.to_string());
                cd.push_str(format!("({}) ", size).as_str());
                cd.push_str(app.selected_item.as_str());
            }

            match md {
                md if md.is_dir() => {
                    let mut cd =  get_current_dir();
                    cd = cd.trim().parse().unwrap();
                    let mut options = fs_extra::dir::CopyOptions::new();
                    options.overwrite = true;
                    let res = fs_extra::dir::copy(app.command.to_string(), cd, &options);
                    result(app, res,  "Copied directory succesfully".to_string());
                },
                md if md.is_file() => {
                    let options = fs_extra::file::CopyOptions::new();
                    let res = fs_extra::file::copy(app.command.to_string(), cd, &options);
                    result(app, res, "Copied file succesfully".to_string());
                },
                _ => {app.title = format!("Error metadata")}
            }
        },
        CommandType::Remove => {
            let res = trash::delete(app.selected_item.to_string());
            match res {
                Ok(_res) => {
                    app.title = format!("Moved {} succesfully to Trash", app.selected_item.to_string());
                    app.items = list_current_dir("-a".to_string());
                    app.view_items = StatefulList::with_items(list_current_dir("-l".to_string()));
                    app.view_items.state.select(Some(0));
                },
                Err(e) => app.title = e.to_string()
            }
        }
        CommandType::Move => {

            let mut cd =  get_current_dir();
            cd = cd.trim().parse().unwrap();
            cd.push_str("/");
            cd.push_str(app.selected_item.as_str());

            let md = Path::new(app.command.as_str());

            if app.command.to_string() == cd && md.is_file() {
                cd.clear();
                cd =  get_current_dir();
                cd = cd.trim().parse().unwrap();
                cd.push_str("/");
                let size = list_current_dir_matches(app.selected_item.to_string());
                cd.push_str(format!("({}) ", size).as_str());
                cd.push_str(app.selected_item.as_str());
            }

            match md {
                md if md.is_dir() => {
                    let mut cd =  get_current_dir();
                    cd = cd.trim().parse().unwrap();
                    let mut options = fs_extra::dir::CopyOptions::new();
                    options.overwrite = true;
                    let res = fs_extra::dir::move_dir(app.command.to_string(), &cd, &options);
                    result(app, res,  "Moved directory succesfully".to_string());
                },
                md if md.is_file() => {
                    let options = fs_extra::file::CopyOptions::new();
                    let res = fs_extra::file::move_file(app.command.to_string(), cd, &options);
                    result(app, res, "Moved file succesfully".to_string());
                },
                _ => {app.title = format!("Error metadata")}
            }
        },
        CommandType::Idle => (),
    }
}
fn result(app: &mut App, res: Result<u64, Error>, on_succes_msg: String) {
    match res {
        Ok(_res) => {
            app.title = on_succes_msg;
            app.items = list_current_dir("-a".to_string());
            app.view_items = StatefulList::with_items(list_current_dir("-l".to_string()));
            app.view_items.state.select(Some(0));
        },
        Err(e) => app.title = format!("Error res {}", e.to_string()),
    }
}
