use ratatui::{buffer::Buffer, layout::{Alignment, Rect}, style::{Style, Stylize}, symbols::border, text::{Span, Text}, widgets::{block::Title, Block, List, ListItem, Padding, Paragraph, Widget}};

#[derive(Debug)]
enum ProjectStatus {
    Finished,
    InProgress,
    Idea
}

#[derive(Debug)]
pub struct Project {
    title: String,
    description: String,
    status: ProjectStatus
}

impl Project {
    fn new(title: String, description: String) -> Self {
        Project {
            title,
            description,
            status: ProjectStatus::Idea
        }
    }

    fn to_string(&self) -> String {
        let status_symbol = match self.status {
            ProjectStatus::Finished => "[✓]",
            ProjectStatus::InProgress => "[-]",
            ProjectStatus::Idea => "[ ]",
        };

        format!("{} {}", status_symbol, self.title)
    }
}

#[derive(Default, Debug)]
pub struct ProjectList {
    projects: Vec<Project>
}

impl ProjectList {
    pub fn add_project(&mut self, title: String, description: String) {
        self.projects.push(Project::new(title, description));
    }
}

impl Widget for &ProjectList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Project List ".bold());

        let block = Block::bordered()
            .title(title.alignment(Alignment::Left))
            .padding(Padding::uniform(1))
            .border_set(border::DOUBLE);

        let items: Vec<ListItem> = self.projects
            .iter()
            .map(|project| {
                ListItem::new(project.to_string())
            })
            .collect();

        List::new(items).block(block).render(area, buf);
    }
}