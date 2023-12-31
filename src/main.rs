mod tasks;
mod ui;

use tasks::{read_tasks_from_file, save_tasks_to_file, Task, TaskStatus};
use ui::{generate_stateful_textbox, generate_task_box, ActiveSection, App, AppState};

use crossterm::event::{self, Event as CEvent, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io;
use std::sync::mpsc;
use std::thread;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};

use tui::widgets::ListItem;
use tui::Terminal;

enum Event<I> {
    Input(I),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("Terminal can run in raw mode.");
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        if let CEvent::Key(key) = event::read().expect("Thread can read user events.") {
            if key.kind == KeyEventKind::Press {
                tx.send(Event::Input(key))
                    .expect("Thread can transmit events.");
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

    let mut backlog_items: Vec<Task>;
    let mut in_progress_items: Vec<Task>;
    let mut done_items: Vec<Task>;

    let _ = match read_tasks_from_file() {
        Ok(vectors) => (backlog_items, in_progress_items, done_items) = vectors,
        Err(_) => {
            backlog_items = Vec::new();
            in_progress_items = Vec::new();
            done_items = Vec::new();
        }
    };

    app.backlog_size = backlog_items.len();
    app.inprogress_size = in_progress_items.len();
    app.done_size = done_items.len();

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

            let inprogress_listitems: Vec<ListItem<'_>> = in_progress_items
                .iter()
                .map(|item| ListItem::new(item.text.to_string()))
                .collect();

            let done_listitems: Vec<ListItem<'_>> = done_items
                .iter()
                .map(|item| ListItem::new(item.text.to_string()))
                .collect();

            let backlog = generate_task_box(&app, backlog_listitems, ActiveSection::Backlog, "Backlog".to_string());
            let inprogress =
                generate_task_box(&app, inprogress_listitems, ActiveSection::InProgress,"In Progress".to_string());
            let done = generate_task_box(&app, done_listitems, ActiveSection::Done, "Done".to_string());
            let textbox = generate_stateful_textbox(&app);

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
                        save_tasks_to_file(backlog_items, in_progress_items, done_items);
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

                    KeyCode::Char('k') => match app.active_selection {
                        ActiveSection::Backlog => {
                            if let Some(selected_idx) = app.backlog_state.selected() {
                                if backlog_items.len() != 0 {
                                    backlog_items[selected_idx]
                                        .change_status(TaskStatus::InProgress);
                                    let popped_task = backlog_items.remove(selected_idx);
                                    in_progress_items.push(popped_task);
                                    app.inprogress_size += 1;
                                    app.backlog_size -= 1;
                                }
                            }
                        }
                        ActiveSection::InProgress => {
                            if let Some(selected_idx) = app.inprogress_state.selected() {
                                if in_progress_items.len() != 0 {
                                    in_progress_items[selected_idx].change_status(TaskStatus::Done);
                                    let popped_task = in_progress_items.remove(selected_idx);
                                    done_items.push(popped_task);
                                    app.done_size += 1;
                                    app.inprogress_size -= 1;
                                }
                            }
                        }
                        ActiveSection::Done => {}
                    },

                    KeyCode::Char('j') => match app.active_selection {
                        ActiveSection::Backlog => {}
                        ActiveSection::InProgress => {
                            if let Some(selected_idx) = app.inprogress_state.selected() {
                                if in_progress_items.len() != 0 {
                                    in_progress_items[selected_idx]
                                        .change_status(TaskStatus::Backlog);
                                    let popped_task = in_progress_items.remove(selected_idx);
                                    backlog_items.push(popped_task);
                                    app.backlog_size += 1;
                                    app.inprogress_size -= 1;
                                }
                            }
                        }
                        ActiveSection::Done => {
                            if let Some(selected_idx) = app.done_state.selected() {
                                if done_items.len() != 0 {
                                    done_items[selected_idx].change_status(TaskStatus::InProgress);
                                    let popped_task = done_items.remove(selected_idx);
                                    in_progress_items.push(popped_task);
                                    app.inprogress_size += 1;
                                    app.done_size -= 1;
                                }
                            }
                        }
                    },

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
            },
        };
    }
}
