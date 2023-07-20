mod tasks;

use crossterm::event::{self, Event as CEvent, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tasks::{Task, TaskStatus};
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph};
use tui::Terminal;

enum Event<I> {
    Input(I),
    Tick,
}
#[derive(Clone, Copy)]
enum ActiveSection {
    Backlog,
    InProgress,
    Done,
}

enum AppState {
    Manage,
    Edit,
}

struct App {
    app_state: AppState,
    active_selection: ActiveSection,
    current_message: String,
}

impl Default for App {
    fn default() -> Self {
        App {
            app_state: AppState::Manage,
            active_selection: ActiveSection::Backlog,
            current_message: String::new(),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("Terminal can run in raw mode.");
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    // Start a thread to update the UI every 200ms
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("Thread polling works.") {
                if let CEvent::Key(key) = event::read().expect("Thread can read user events.") {
                    if key.kind == KeyEventKind::Press {
                        tx.send(Event::Input(key))
                            .expect("Thread can transmit events.");
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Default
    let mut app = App::default();
    let columns = [
        ActiveSection::Backlog,
        ActiveSection::InProgress,
        ActiveSection::Done,
    ];
    let mut colptr: usize = 0;

    let mut tasks: Vec<Task> = Vec::new();
    let mut backlog_items: Vec<ListItem> = Vec::new();
    let mut in_progress_items: Vec<ListItem> = Vec::new();
    let mut done_items: Vec<ListItem> = Vec::new();

    update(
        &tasks,
        &mut backlog_items,
        &mut in_progress_items,
        &mut done_items,
    );

    loop {
        // Scroll to current column
        app.active_selection = columns[colptr];

        terminal.draw(|canvas| {
            let size = canvas.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
                .vertical_margin(0)
                .split(size);

            let body_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints(
                    [
                        Constraint::Ratio(1, 3),
                        Constraint::Ratio(1, 3),
                        Constraint::Ratio(1, 3),
                    ]
                    .as_ref(),
                )
                .vertical_margin(0)
                .split(chunks[0]);

            let backlog = List::new(backlog_items.as_ref())
                .block(
                    Block::default()
                        .title(" Backlog ")
                        .borders(Borders::ALL)
                        .border_type(match app.active_selection {
                            ActiveSection::Backlog => BorderType::Double,
                            _ => BorderType::Plain,
                        }),
                )
                .highlight_style(Style::default())
                .highlight_symbol(">>")
                .style(Style::default().fg(Color::White));

            let inprogress = List::new(in_progress_items.as_ref())
                .block(
                    Block::default()
                        .title(" In Progress ")
                        .borders(Borders::ALL)
                        .border_type(match app.active_selection {
                            ActiveSection::InProgress => BorderType::Double,
                            _ => BorderType::Plain,
                        }),
                )
                .highlight_style(Style::default())
                .highlight_symbol(">>")
                .style(Style::default().fg(Color::White));

            let done = List::new(done_items.as_ref())
                .block(
                    Block::default()
                        .title(" Done ")
                        .borders(Borders::ALL)
                        .border_type(match app.active_selection {
                            ActiveSection::Done => BorderType::Double,
                            _ => BorderType::Plain,
                        }),
                )
                .highlight_style(Style::default())
                .highlight_symbol(">>")
                .style(Style::default().fg(Color::White));

            let textbox = Paragraph::new(app.current_message.as_ref()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double),
                );
                
            match app.app_state {
                    AppState::Edit => {
                    canvas.set_cursor(
                        // Put cursor past the end of the input text
                        chunks[1].x + app.current_message.len() as u16 + 1,
                        // Move one line down, from the border to the input line
                        chunks[1].y + 1,
                    )
                },
                
                AppState::Manage => {}
            }

            canvas.render_widget(backlog, body_chunks[0]);
            canvas.render_widget(inprogress, body_chunks[1]);
            canvas.render_widget(done, body_chunks[2]);
            canvas.render_widget(textbox, chunks[1]);
        })?;

        // Listen for user input
        match app.app_state {
            // Management mode
            AppState::Manage => match rx.recv()? {
                Event::Input(event) => match event.code {
                    KeyCode::Char('q') => {
                        // On quit, disable the terminal and give back control.
                        disable_raw_mode()?;
                        terminal.clear()?;
                        terminal.show_cursor()?;
                        return Ok(());
                    }

                    KeyCode::Left => {
                        if colptr != 0 {
                            colptr -= 1
                        };
                    }
                    KeyCode::Right => {
                        if colptr != columns.len() - 1 {
                            colptr += 1
                        };
                    }
                    KeyCode::Char('i') => {
                        app.app_state = AppState::Edit;
                    },
                    _ => {}
                },

                Event::Tick => {}
            },

            // Edit / insert mode
            AppState::Edit => match rx.recv()? {
                Event::Input(event) => match event.code {
                    KeyCode::Char(c) => {
                        app.current_message.push(c);
                    }
                    KeyCode::Backspace => {
                        app.current_message.pop();
                    }
                    KeyCode::Esc => {
                        app.app_state = AppState::Manage;
                    }
                    KeyCode::Enter => {
                        let msg_text = app.current_message.drain(..).collect();
                        let new_task: Task = Task::create_new_task(msg_text, TaskStatus::Backlog);
                        tasks.push(new_task);
                        update(&tasks, &mut backlog_items, &mut in_progress_items, &mut done_items);
                    }
                    _ => {}
                },

                Event::Tick => {}
            },
        };
    }
}

fn update(
    tasks: &Vec<Task>,
    backlog_items: &mut Vec<ListItem>,
    in_progress_items: &mut Vec<ListItem>,
    done_items: &mut Vec<ListItem>,
) {
    backlog_items.clear();
    in_progress_items.clear();
    done_items.clear();
    for task in tasks {
        match task.get_status() {
            TaskStatus::Backlog => backlog_items.push(task.to_list_item()),
            TaskStatus::InProgress => in_progress_items.push(task.to_list_item()),
            TaskStatus::Done => done_items.push(task.to_list_item()),
        }
    }
}
