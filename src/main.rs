use movie_mover::app::App;
use movie_mover::event::{Event, EventHandler};
use movie_mover::handler::handle_key_events;
use movie_mover::tui::Tui;
use std::io;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an application.

    // save_config()?;
    // panic!();

    let config = movie_mover::config::read_config()?;
    let mut app = App::new(config).map_err(|err| err.content)?;

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;

    Ok(())
}
