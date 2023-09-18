mod app;
mod event;
mod handler;
mod tui;
mod ui;
mod system;

use std::io;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use crate::app::{App, AppResult};
use crate::event::{Event, EventHandler};
use crate::handler::handle_key_events;
use crate::tui::Tui;

fn main() -> AppResult<()> {
    // Create an application.
    let mut app = App::new();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(500);
    let mut tui = Tui::new(terminal, events);
    let mut refresh = false;
    tui.init()?;

    // Start the main loop.
    while app.running {
        if refresh {
            // Render the user interface.
            tui.draw(&mut app)?;
        }else {
            refresh = true
        }

        // Handle events.
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => { refresh = handle_key_events(key_event, &mut app)? },
            Event::Mouse(_) => { refresh = false;}
            Event::Resize(_, _) => { refresh = false; }
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}