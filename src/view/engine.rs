use crate::view::app::TuiApp;
use crate::view::terminal::{TerminalGuard, poll_stdin, read_stdin_char};
use crate::view::tui::{Rect, ScreenBuffer, get_terminal_size};
use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook::flag;
use std::io::{Error, Write, stdout};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub struct TuiEngine {
    screen_buffer: ScreenBuffer,
    flush_buffer: String,
    term_signal: Arc<AtomicBool>,
}

impl TuiEngine {
    pub fn new() -> Self {
        let (cols, rows) = get_terminal_size();
        let term_signal = Arc::new(AtomicBool::new(false));
        let _ = flag::register(SIGINT, Arc::clone(&term_signal));
        let _ = flag::register(SIGTERM, Arc::clone(&term_signal));

        TuiEngine {
            screen_buffer: ScreenBuffer::new(cols, rows),
            flush_buffer: String::with_capacity(cols * rows * 4),
            term_signal,
        }
    }

    pub fn run(&mut self, app: &mut impl TuiApp, _refresh_time: u64) -> Result<(), Error> {
        let _guard = TerminalGuard::new()?;
        let mut stdout = stdout();

        loop {
            if self.term_signal.load(Ordering::Relaxed) {
                break;
            }

            let (cols, rows) = get_terminal_size();
            self.screen_buffer.resize(cols, rows);
            self.screen_buffer.clear();

            app.update();

            let area = Rect::new(0, 0, cols, rows);
            app.draw(&mut self.screen_buffer, area);

            self.screen_buffer.render(&mut self.flush_buffer);
            write!(stdout, "{}", self.flush_buffer)?;
            stdout.flush()?;

            // Poll stdin with a short 50ms timeout for instant key responsiveness
            // and smooth background task updates
            match poll_stdin(Duration::from_millis(50)) {
                Ok(true) => {
                    if let Ok(key) = read_stdin_char()
                        && !app.handle_input(key)
                    {
                        break;
                    }
                }
                Ok(false) => {}
                Err(_) => continue,
            }
        }

        Ok(())
    }
}
