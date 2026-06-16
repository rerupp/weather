//! The terminal UI runner.
//!
//! The console runs an [Application]. It is responsible for managing the terminal,
//! processing events, and rendering the screen.
//!
use super::*;
use crossterm::event::Event;
use crossterm::{event, execute, terminal};
use std::{io::Stdout, time::Duration};
use toolslib::stopwatch::StopWatch;

/// The result of an event passed onto the [Application].
///
#[derive(Debug)]
pub enum ApplicationResult {
    /// The application is in an exit state.
    Exit,
    /// The application had an unrecoverable error.
    Error(String),
    /// The application is requesting a specific poll interval.
    Poll(Option<usize>),
}
impl std::fmt::Display for ApplicationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// The console application interface.
pub trait Application {
    /// Consume a key pressed event.
    ///
    /// # Arguments
    ///
    /// * `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) type event.
    ///
    fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<ApplicationResult>;
    /// Requests the application to update the terminal screen.
    ///
    /// # Arguments
    ///
    /// * `frame` is the terminal screen view the application will use to render the UI.
    ///
    fn render(&self, frame: &mut Frame);
    /// Notify the application the terminal screen has been resized.
    ///
    /// # Arguments
    ///
    /// * `size` is the new terminal screen size.
    ///
    #[allow(unused_variables)]
    fn resized(&mut self, size: Size) {
        ()
    }
}

/// The console default poll interval.
const DEFAULT_POLL_INTERVAL_MS: u64 = 600_000;

/// The terminal UI runner.
pub struct Console {
    /// The internal [ratatui] terminal state.
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// The time to wait for a key event before forcing a screen refresh.
    poll_duration: Duration,
}
impl Console {
    /// Create a new instance of the console.
    ///
    /// This will fail if there is a problem creating the `ratatui` [CrosstermBackend].
    ///
    pub fn new() -> Result<Self> {
        let stdout = stdout();
        Ok(Self {
            terminal: Terminal::new(CrosstermBackend::new(stdout))?,
            poll_duration: Duration::from_millis(DEFAULT_POLL_INTERVAL_MS),
        })
    }
    /// Initialize the terminal, run the application, and restore the terminal when complete.
    ///
    /// # Arguments
    ///
    /// * `application` is what it sounds like.
    ///
    pub fn run(&mut self, mut application: impl Application) -> Result<()> {
        // set up the terminal
        terminal::enable_raw_mode()?;
        execute!(self.terminal.backend_mut(), terminal::EnterAlternateScreen)?;
        // sit in an applications key event loop
        let mut elapsed_draw: Option<StopWatch> = None;
        let mut elapsed_key_event: Option<StopWatch> = None;
        let mut redraw = true;
        loop {
            if redraw {
                redraw = false;
                let mut elapsed = StopWatch::start_new();
                self.terminal.draw(|frame| {
                    log_render!("Console", "\nsize {:?}", frame.area());
                    application.render(frame)
                })?;
                elapsed.stop();
                elapsed_draw.replace(elapsed);
            }
            // The way this loop works this is where you need to log
            match (elapsed_draw.take(), elapsed_key_event.take()) {
                (Some(draw_time), None) => log::trace!("draw time {}", draw_time),
                (None, Some(key_event_time)) => log::trace!("event time {}", key_event_time),
                (Some(draw_time), Some(event_time)) => {
                    log::trace!("key event {} render {}", event_time, draw_time);
                }
                _ => {}
            }
            match event::poll(self.poll_duration)? {
                false => redraw = true,
                true => match event::read()? {
                    Event::Key(key_event) => {
                        if key_event.kind == KeyEventKind::Press {
                            redraw = true;
                            let mut elapsed = StopWatch::start_new();
                            log_key_pressed!("Console", "\n{:?}", key_event);
                            if let ControlFlow::Break(result) = application.key_pressed(key_event) {
                                match result {
                                    ApplicationResult::Exit => break,
                                    ApplicationResult::Error(err) => Err(Error::from(err))?,
                                    ApplicationResult::Poll(poll_ms) => {
                                        self.poll_duration = Duration::from_millis(match poll_ms {
                                            None => DEFAULT_POLL_INTERVAL_MS,
                                            Some(ms) => ms as u64,
                                        });
                                    }
                                }
                            }
                            elapsed.stop();
                            elapsed_key_event.replace(elapsed);
                        }
                    }
                    Event::Resize(columns, rows) => {
                        redraw = true;
                        application.resized(Size { width: columns, height: rows });
                    }
                    unhandled_event => {
                        log::trace!("{:?}", unhandled_event)
                    }
                },
            }
        }
        // drop will restore the terminal state
        Ok(())
    }
}
impl Drop for Console {
    /// In order to catch panics [Drop] is implemented to allow the terminal state to be
    /// restored.
    fn drop(&mut self) {
        match terminal::disable_raw_mode() {
            Ok(_) => match execute!(self.terminal.backend_mut(), terminal::LeaveAlternateScreen) {
                Ok(_) => match self.terminal.show_cursor() {
                    Ok(_) => (),
                    Err(err) => log::error!("Failed to show the terminal cursor ({}).", err),
                },
                Err(err) => log::error!("Did not leave terminals alternate screen ({}).", err),
            },
            Err(err) => log::error!("Did not disable terminal raw mode ({}).", err),
        }
    }
}
