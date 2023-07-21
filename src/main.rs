mod tasks;
use tasks::{Task, TaskStatus};

use crossterm::event::{self, Event as CEvent, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tui::backend::CrosstermBackend;
use tui::layout::Alignment;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};
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
    backlog_size: usize,
    inprogress_size: usize,
    done_size: usize,
    current_message: String,
    current_selection_idx: usize,
    backlog_state: ListState,
    inprogress_state: ListState,
    done_state: ListState,
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

    let mut backlog_items: Vec<Task> = Vec::new();
    let mut in_progress_items: Vec<Task> = Vec::new();
    let mut done_items: Vec<Task> = Vec::new();

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

            let backlog_listitems: Vec<ListItem<'_>> = backlog_items
                .iter()
                .map(|item| ListItem::new(item.text.to_string()))
                .collect();
            let backlog = List::new(backlog_listitems.as_ref())
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
                .highlight_symbol("->")
                .style(Style::default().fg(Color::White));

            let inprogress_listitems: Vec<ListItem<'_>> = in_progress_items
                .iter()
                .map(|item| ListItem::new(item.text.to_string()))
                .collect();
            let inprogress = List::new(inprogress_listitems.as_ref())
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
                .highlight_symbol("->")
                .style(Style::default().fg(Color::White));

            let done_listitems: Vec<ListItem<'_>> = done_items
                .iter()
                .map(|item| ListItem::new(item.text.to_string()))
                .collect();
            let done = List::new(done_listitems.as_ref())
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
                .highlight_symbol("->")
                .style(Style::default().fg(Color::White));

            let textbox = match app.app_state {
                AppState::Manage => Paragraph::new("Use arrow keys to move around, <i> to go to edit mode, <del> to delete a task and <q> to quit.")
                .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain),)
                .alignment(Alignment::Center),
            

                AppState::Edit => {
                    Paragraph::new(app.current_message.as_ref()).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Plain),
                    )
                },
            };

            match app.app_state {
                AppState::Edit => {
                    canvas.set_cursor(
                        // Put cursor past the end of the input text
                        chunks[1].x + app.current_message.len() as u16 + 1,
                        // Move one line down, from the border to the input line
                        chunks[1].y + 1,
                    )
                }

                AppState::Manage => match app.active_selection {
                    ActiveSection::Backlog => {
                        app.backlog_state.select(Some(app.current_selection_idx));
                    }
                    ActiveSection::InProgress => {
                        app.inprogress_state.select(Some(app.current_selection_idx))
                    }
                    ActiveSection::Done => {
                        app.done_state.select(Some(app.current_selection_idx));
                    }
                },
            }

            canvas.render_stateful_widget(backlog, body_chunks[0], &mut app.backlog_state);
            canvas.render_stateful_widget(inprogress, body_chunks[1], &mut app.inprogress_state);
            canvas.render_stateful_widget(done, body_chunks[2], &mut app.done_state);
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
                    KeyCode::Down => {
                        let upper_limit;
                        match app.active_selection {
                            ActiveSection::Backlog => upper_limit = app.backlog_size,
                            ActiveSection::InProgress => upper_limit = app.inprogress_size,
                            ActiveSection::Done => upper_limit = app.done_size,
                        }
                        if upper_limit != 0 && app.current_selection_idx < upper_limit - 1 {
                            app.current_selection_idx += 1;
                        }
                    }
                    KeyCode::Up => {
                        if app.current_selection_idx != 0 {
                            app.current_selection_idx -= 1;
                        }
                    }
                    KeyCode::Left => {
                        if colptr != 0 {
                            colptr -= 1;
                            app.current_selection_idx = 0;
                        };
                    }
                    KeyCode::Right => {
                        if colptr != columns.len() - 1 {
                            colptr += 1;
                            app.current_selection_idx = 0;
                        };
                    }
                    KeyCode::Char('i') => {
                        app.app_state = AppState::Edit;
                    }

                    KeyCode::Delete => match app.active_selection {
                        ActiveSection::Backlog => {
                            if let Some(selected_idx) = app.backlog_state.selected() {
                                if backlog_items.len() != 0 {
                                    backlog_items.remove(selected_idx);
                                    app.backlog_size -= 1;
                                }
                                if app.current_selection_idx != 0 {
                                    app.current_selection_idx -= 1;
                                }
                            }
                        }
                        ActiveSection::InProgress => {
                            if let Some(selected_idx) = app.inprogress_state.selected() {
                                if in_progress_items.len() != 0 {
                                    in_progress_items.remove(selected_idx);
                                    app.inprogress_size -= 1;
                                }
                                if app.current_selection_idx != 0 {
                                    app.current_selection_idx -= 1;
                                }
                            }
                        }
                        ActiveSection::Done => {
                            if let Some(selected_idx) = app.done_state.selected() {
                                if done_items.len() != 0 {
                                    done_items.remove(selected_idx);
                                    app.done_size -= 1;
                                }
                                if app.current_selection_idx != 0 {
                                    app.current_selection_idx -= 1;
                                }
                            }
                        }
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
                        match app.active_selection {
                            ActiveSection::Backlog => {
                                backlog_items
                                    .push(Task::create_new_task(msg_text, TaskStatus::Backlog));
                                app.backlog_size += 1;
                            }
                            ActiveSection::InProgress => {
                                in_progress_items
                                    .push(Task::create_new_task(msg_text, TaskStatus::InProgress));
                                app.inprogress_size += 1;
                            }
                            ActiveSection::Done => {
                                done_items.push(Task::create_new_task(msg_text, TaskStatus::Done));
                                app.done_size += 1;
                            }
                        }
                    }
                    _ => {}
                },

                Event::Tick => {}
            },
        };
    }
}
