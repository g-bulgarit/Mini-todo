use crossterm::event::{self, Event as CEvent, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, BorderType, Borders, List, ListItem};
use tui::Terminal;

enum Event<I> {
    Input(I),
    Tick,
}

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
}

impl Default for App {
    fn default() -> Self {
        App {
            app_state: AppState::Manage,
            active_selection: ActiveSection::Backlog,
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
    let mut colptr: i32 = 0;

    loop {
        terminal.draw(|canvas| {
            let size = canvas.size();
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                    ]
                    .as_ref(),
                )
                .split(size);

            // For testing
            let backlog_items = [
                ListItem::new("Finish styling"),
                ListItem::new("Eat"),
                ListItem::new("Go to bed"),
            ];
            let in_progress_items = [ListItem::new("Implement logic")];
            let done_items = [ListItem::new("Get basic code running")];
            // --------

            let backlog = List::new(backlog_items)
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

            let inprogress = List::new(in_progress_items)
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

            let done = List::new(done_items)
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

            canvas.render_widget(backlog, chunks[0]);
            canvas.render_widget(inprogress, chunks[1]);
            canvas.render_widget(done, chunks[2]);
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
                    KeyCode::Char('b') => app.active_selection = ActiveSection::Backlog,
                    KeyCode::Char('p') => app.active_selection = ActiveSection::InProgress,
                    KeyCode::Char('d') => app.active_selection = ActiveSection::Done,
                    KeyCode::Left => {
                        colptr = colptr - 1;
                        println!("{}", colptr);
                    }
                    KeyCode::Right => {
                        colptr = colptr + 1;
                        println!("{}", colptr);
                    }
                    KeyCode::Char('i') => app.app_state = AppState::Edit,
                    _ => {}
                },

                Event::Tick => {}
            },

            // Edit / insert mode
            AppState::Edit => match rx.recv()? {
                Event::Input(event) => match event.code {
                    KeyCode::Char('q') => {
                        // On quit, disable the terminal and give back control.
                        disable_raw_mode()?;
                        terminal.clear()?;
                        terminal.show_cursor()?;
                        return Ok(());
                    }
                    KeyCode::Esc => {
                        app.app_state = AppState::Manage;
                    }
                    _ => {}
                },

                Event::Tick => {}
            },
        };
    }
}
