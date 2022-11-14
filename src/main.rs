// use anyhow::Context;
use argh::FromArgs;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    time::{Instant, Duration},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

mod app;
mod rtt;
mod ui;

#[derive(FromArgs)]
/// Track a train
struct RttTui {
    /// time in ms between two ticks (default 250)
    #[argh(option, default = "100")]
    tick_rate: u64,

    /// time in seconds between remote API updates (default 30)
    #[argh(option, default = "30")]
    refresh_rate: u64,

    /// departure time, ie 0830
    #[argh(positional)]
    departs: String,

    /// three letter source station (crs), ie EDB, or the TIPLOC code source station
    #[argh(positional)]
    source: String,

    /// three letter destination station (crs), ie KGX, or the TIPLOC code destination station
    #[argh(positional)]
    dest: String
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: app::App, tick_rate: Duration) -> anyhow::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui::draw(f, &mut app, last_tick))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Similar to app::App::on_key we're going to allow single_match here
                // as this is the place we'd include support for non-letter key matches
                #[allow(clippy::single_match)]
                match key.code {
                    KeyCode::Char(c) => app.on_key(c),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick().await?;
            last_tick = Instant::now();
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts: RttTui = argh::from_env();
    let tick_rate = Duration::from_millis(opts.tick_rate);
    let refresh_rate = Duration::from_secs(opts.refresh_rate);

    println!("Train Tracking, Data Provided by Realtime Trains (realtimetrains.co.uk)");
    println!("Searching for the {} from {} to {}", opts.departs, opts.source, opts.dest);

    let mut app = app::App::new(refresh_rate);
    app.load_destination(opts.dest).await?;
    app.find_service(opts.source, opts.departs).await?;
    app.refresh_service().await?;

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    
    run_app(&mut terminal, app, tick_rate).await?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    Ok(())
}
