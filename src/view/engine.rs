use std::io::{Write, Error, stdout};
use std::time::Duration;
use crate::view::tui::{clear_screen, get_terminal_size};
use crate::view::terminal::{enable_raw_mode, disable_raw_mode, poll_stdin, read_stdin_char};
use crate::view::app::TuiApp;

const MARGIN: usize = 2;

pub struct TuiEngine {
    render_buffer: String,
}

impl TuiEngine {
    pub fn new() -> Self {
        TuiEngine {
            render_buffer: String::with_capacity(4096),
        }
    }

    pub fn run(&mut self, app: &mut impl TuiApp) -> Result<(), Error> {
        let mut stdout = stdout();
        enable_raw_mode()?;
        
        clear_screen(&mut self.render_buffer);
        write!(stdout, "{}", self.render_buffer)?;
        stdout.flush()?;
        self.render_buffer.clear();
        
        loop {
            let (cols, _rows) = get_terminal_size();
            clear_screen(&mut self.render_buffer);
            
            let dynamic_width = cols.saturating_sub(2 * MARGIN);
            
            app.update();
            app.draw(&mut self.render_buffer, dynamic_width);
            
            write!(stdout, "{}", self.render_buffer)?;
            stdout.flush()?;
            self.render_buffer.clear();
            
            // Handle user input with a timeout to allow for periodic updates
            match poll_stdin(Duration::from_millis(100)) {
                Ok(true) => {
                    if let Ok(key) = read_stdin_char() && !app.handle_input(key) {
                        break;
                    }
                }
                Ok(false) => {}
                Err(_) => continue,
            }
        }
        disable_raw_mode()?;
        Ok(())
    }
}
