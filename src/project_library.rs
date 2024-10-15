use core::fmt;
use std::{fs, path::PathBuf};

use color_eyre::eyre::Result;
use ratatui::{layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Style, Stylize}, symbols, text::{Line, Span, Text}, widgets::{block::Title, Block, BorderType, Borders, List, ListItem, Padding, Paragraph, Widget}};
use serde::{Deserialize, Serialize};

pub enum CycleDirection {
    Up,
    Down
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum ProjectStatus {
    Finished,
    InProgress,
    Idea,
    Paused,
}

impl ProjectStatus {
    pub fn cycle(&self, direction: CycleDirection) -> Self {
        match direction {
            CycleDirection::Up => match self {
                Self::Finished => Self::Paused,
                Self::InProgress => Self::Finished,
                Self::Idea => Self::InProgress,
                Self::Paused => Self::Idea
            },
            CycleDirection::Down => match self {
                Self::Finished => Self::InProgress,
                Self::InProgress => Self::Idea,
                Self::Idea => Self::Paused,
                Self::Paused => Self::Finished
            }
        }

    }

    pub fn to_symbol(&self) -> Span<'_> {
        match self {
            Self::Finished => "✔".green(),
            Self::InProgress => "-".yellow(),
            Self::Paused => "X".red(),
            Self::Idea => "!".white()
        }
    }
}

impl fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self {
            Self::Finished => "Finished",
            Self::InProgress => "In Progress",
            Self::Paused => "Paused",
            Self::Idea => "Idea"
        };

        write!(f, "{}", status)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Project {
    name: String,
    description: String,
    status: ProjectStatus,
}

impl Project {
    pub fn new(name: &str, description: &str) -> Self {
        Project {
            name: name.to_string(),
            description: description.to_string(),
            status: ProjectStatus::Idea
        }
    }

    pub fn set_status(&mut self, status: ProjectStatus) {
        self.status = status;
    }

    pub fn cycle_status(&mut self, direction: CycleDirection) {
        self.status = self.status.cycle(direction);
    }

    pub fn to_list_item(&self) -> ListItem {
        let symbol = self.status.to_symbol();
        let text = Text::from(format!("{} {}", symbol.content, self.name));
        ListItem::new(text).style(symbol.style)
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct ProjectLibrary {
    projects: Vec<Project>,
    #[serde(skip_serializing, skip_deserializing)]
    selected_project_index: usize,
    #[serde(skip_serializing, skip_deserializing)]
    library_file_path: PathBuf
}

impl ProjectLibrary {
    pub fn from_file(file_path: &PathBuf) -> Result<Self> {
        let file_string = fs::read_to_string(file_path)?;
        let mut parsed_lib = toml::from_str::<ProjectLibrary>(&file_string)?;
        parsed_lib.library_file_path = file_path.clone();
        Ok(parsed_lib)
    }

    pub fn save(&self) -> Result<()> {
        let to_string = toml::to_string_pretty(self)?;
        fs::write(&self.library_file_path, to_string)?;
        Ok(())
    }

    pub fn add_project(&mut self, project: Project) -> &Self {
        self.projects.push(project);
        self
    }

    pub fn cycle_selected_project(&mut self, direction: CycleDirection) {
        match direction {
            CycleDirection::Down => {
                if self.selected_project_index >= self.projects.len()-1 {
                    self.selected_project_index = 0;
                } else {
                    self.selected_project_index += 1;
                }
            },
            CycleDirection::Up => {
                if self.selected_project_index == 0 {
                    self.selected_project_index = self.projects.len() - 1;
                } else {
                    self.selected_project_index -= 1;
                }
            }
        }
    }

    pub fn cycle_selected_project_status(&mut self, direction: CycleDirection) {
        if let Some(project) = self.projects.get_mut(self.selected_project_index) {
            project.cycle_status(direction);
        }
    }
}

impl Widget for &ProjectLibrary {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(70)
            ]).split(area);
        
        let project_list_title = Title::from(" Projects ".bold());
        let project_list_block = Block::new()
                .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
                .title(project_list_title.alignment(Alignment::Left))
                .padding(Padding::horizontal(1))
                .border_type(BorderType::Double);

        let _ = build_project_list(&self.projects, self.selected_project_index).block(project_list_block).render(chunks[0], buf);
        
        let project_details_title = Title::from(" Project Details ".bold());
        let project_details_block = Block::bordered()
                .title(project_details_title.alignment(Alignment::Left))
                .padding(Padding::horizontal(1))
                .border_set(
                    symbols::border::Set {
                        top_left: symbols::line::DOUBLE.horizontal_down,
                        bottom_left: symbols::line::DOUBLE.horizontal_up,
                        ..symbols::border::DOUBLE
                    }
                );

        if self.selected_project_index < self.projects.len() {
            let selected_project = &self.projects[self.selected_project_index];

            let text = Text::from(vec![
                Line::from(Span::from(format!("Project Name: {}", selected_project.name))),
                Line::from(Span::from(format!("Status: {}", selected_project.status))),
                Line::from(Span::from("Description:")),
                Line::from(Span::from(&selected_project.description)),
            ]);

            let _ = Paragraph::new(text).block(project_details_block).render(chunks[1], buf);
        } else {
            project_details_block.render(chunks[1], buf);
        }
    }
}

fn build_project_list(projects: &Vec<Project>, selected_project_index: usize) -> List {
    let list_items = projects.iter().enumerate().map(|(index, project)| {
        let item = project.to_list_item();
        if index == selected_project_index { 
            item.bold().bg(Color::Blue)
        } else {
            item
        }
    }
    ).collect::<Vec<ListItem>>();

    List::new(list_items)
        .highlight_style(Style::default().bg(Color::Blue).bold())
}