#[path = "./tasks.rs"]
mod tasks;
use tui::layout::Alignment;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};

#[derive(Clone, Copy, PartialEq)]
pub enum ActiveSection {
    Backlog,
    InProgress,
    Done,
}

pub enum AppState {
    Manage,
    Edit,
}

pub struct App {
    pub app_state: AppState,
    pub active_selection: ActiveSection,
    pub backlog_size: usize,
    pub inprogress_size: usize,
    pub done_size: usize,
    pub current_message: String,
    pub current_selection_idx: usize,
    pub backlog_state: ListState,
    pub inprogress_state: ListState,
    pub done_state: ListState,
}

impl Default for App {
    fn default() -> Self {
        App {
            app_state: AppState::Manage,
            active_selection: ActiveSection::Backlog,
            current_message: String::new(),
            current_selection_idx: 0,
            backlog_size: 0,
            inprogress_size: 0,
            done_size: 0,
            backlog_state: ListState::default(),
            inprogress_state: ListState::default(),
            done_state: ListState::default(),
        }
    }
}

pub fn generate_stateful_textbox(app: &App) -> Paragraph<'static> {
    match app.app_state {
        AppState::Manage => Paragraph::new(
            "<i> to insert, <j, k> to move task, <del> to delete a task and <q> to quit.",
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain),
        )
        .alignment(Alignment::Center),

        AppState::Edit => Paragraph::new(app.current_message.clone()).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain),
        ),
    }
}

pub fn generate_task_box<'a>(app: &App, item_list: Vec<ListItem<'a>>, selector: ActiveSection, title: String) -> List<'a> {
    List::new(item_list.as_ref())
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(match selector == app.active_selection {
                    true => BorderType::Double,
                    false => BorderType::Plain,
                }),
        )
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("-> ")
        .style(match selector == app.active_selection {
            true => Style::default().fg(Color::Cyan),
            false => Style::default(),
        })
}
