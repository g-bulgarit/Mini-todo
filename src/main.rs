use crossterm::event::{self, Event as CEvent, KeyCode};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, BorderType, Borders};
use tui::Terminal;

enum Event<I> {
    Input(I),
    Tick,
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
                    tx.send(Event::Input(key))
                        .expect("Thread can transmit events.");
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

    loop {
        terminal.draw(|canvas| {
            let size = canvas.size();
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(2)
                .constraints(
                    [
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                    ]
                    .as_ref(),
                )
                .split(size);

            let backlog = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Backlog")
                .border_type(BorderType::Plain);

            let inprogress = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("In Progress")
                .border_type(BorderType::Plain);

            let done = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Done")
                .border_type(BorderType::Plain);

            canvas.render_widget(backlog, chunks[0]);
            canvas.render_widget(inprogress, chunks[1]);
            canvas.render_widget(done, chunks[2]);
        })?;

        // Listen for user input
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    // On quit, disable the terminal and give back control.
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                },
                _ => {}
                
            }
            Event::Tick => {}
        }
    }
    Ok(())
}
