use std::{fs, io, path::PathBuf};

use color_eyre::eyre::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use project_library::{CycleDirection, ProjectLibrary};
use ratatui::{layout::{Constraint, Direction, Layout}, style::Stylize, text::{Line, Text}, widgets::{Block, BorderType, Paragraph}, DefaultTerminal, Frame};
mod project_library;

#[derive(PartialEq)]
enum AppState {
    MainView,
    DeletingProject,
    AddingProject,
    Exiting
}

struct App {
    project_library: ProjectLibrary,
    state: AppState
}

impl App {
    fn new(project_library: ProjectLibrary) -> Self {
        App { project_library, state: AppState::MainView }
    }

    fn exit(&mut self) -> Result<()> {
        self.project_library.save()
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.state != AppState::Exiting {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        self.exit()?;
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3)
            ]).split(frame.area());

        frame.render_widget(&self.project_library, chunks[0]);

        let instructions_block = Block::bordered()
                .border_type(BorderType::Double);
        
        frame.render_widget(Paragraph::new(Text::from(Line::from(vec![
            " Scroll Up ".into(),
            "<Up> or <k>".blue().bold(),
            " Scroll Down ".into(),
            "<Down> or <j>".blue().bold(),
            " Add Project ".into(),
            "<A>".blue().bold(),
            " Quit ".into(),
            "<q>".blue().bold()
        ]))).block(instructions_block), chunks[1]);
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.state = AppState::Exiting,
            KeyCode::Up | KeyCode::Char('k') => self.project_library.cycle_selected_project(CycleDirection::Up),
            KeyCode::Down | KeyCode::Char('j') => self.project_library.cycle_selected_project(CycleDirection::Down),
            KeyCode::Char(' ') => self.project_library.cycle_selected_project_status(CycleDirection::Up),
            KeyCode::Char('A') => self.state = AppState::AddingProject,
            KeyCode::Char('D') => self.state = AppState::DeletingProject,
            KeyCode::Esc => self.state = AppState::MainView,
            _ => {}
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
           Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            self.handle_key_event(key_event)
           } 
            _ => {}
        };
        Ok(())
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let config_dir = if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config")
    } else {
        PathBuf::from("~/.config/projectlib")
    };

    if !config_dir.try_exists()? {
        fs::create_dir_all(&config_dir)?;
    }

    let library_file_path = config_dir.join("config.toml");
    if !library_file_path.try_exists()? {
        fs::write(&library_file_path, toml::to_string_pretty(&ProjectLibrary::default())?)?;
    }

    let project_library = ProjectLibrary::from_file(&library_file_path)?;

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = App::new(project_library).run(&mut terminal);
    ratatui::restore();
    app_result.wrap_err("App Error")
}