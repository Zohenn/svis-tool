mod terminal;
mod theme;
mod tui;
mod utils;

use anyhow::{Error, Result};
use clap::{arg, builder::ArgPredicate, Arg, Command};
use core::analyze_path;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tui::{run_tui_app, App};
use ui::terminal::{get_default_styles, print_file_info};

fn main() -> Result<()> {
    let matches = Command::new("svis-tool")
        .arg(arg!(-t --tui "run as tui app").default_value("true").default_value_if(
            "simple",
            ArgPredicate::IsPresent,
            None,
        ))
        .arg(arg!(-s --simple "run without tui").requires("path"))
        .arg(Arg::new("path").short('p').help("path to scan files for"))
        .get_matches();

    let path = matches.get_one::<String>("path");
    match matches.get_one::<bool>("tui") {
        Some(_) => run_tui(path.map(|x| x.as_str())),
        None => run_simple(&path.unwrap()),
    }
}

fn run_tui(path: Option<&str>) -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic| {
        disable_raw_mode().unwrap();
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
        original_hook(panic);
    }));

    // create app and run it
    let app = App::default();
    let res = run_tui_app(&mut terminal, app, path);

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_simple(path: &str) -> Result<()> {
    let styles = get_default_styles();
    let mut files_checked = 0u32;
    let mut files_with_errors: Vec<(String, Error)> = vec![];

    analyze_path(path, |file, result| {
        files_checked += 1;
        match result {
            Ok(info) => print_file_info(&info),
            Err(err) => files_with_errors.push((file.to_owned(), err)),
        }
    })?;

    for (file, err) in files_with_errors {
        println!(
            "{} Error when parsing file {}, make sure the sourcemap is correct:\n- {}",
            styles.error.apply_to("!"),
            styles.file.apply_to(file),
            err,
        );
    }

    println!("Files checked: {}", styles.highlight.apply_to(files_checked));

    Ok(())
}
