use libc::{termios, tcgetattr, tcsetattr, TCSANOW, STDIN_FILENO, ECHO, ICANON};
use libc::{poll, pollfd, POLLIN};
use std::io::{self, Read, Write};
use std::mem;
use std::time::Duration;

static mut ORIGINAL_TERMIOS: Option<termios> = None;

pub fn enable_raw_mode() -> Result<(), std::io::Error> {
    unsafe {
        let mut raw = mem::zeroed();
        if tcgetattr(STDIN_FILENO, &mut raw) != 0 {
            return Err(std::io::Error::last_os_error());
        }

        ORIGINAL_TERMIOS = Some(raw);

        raw.c_lflag &= !(ICANON | ECHO);

        if tcsetattr(STDIN_FILENO, TCSANOW, &raw) != 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }
}

pub fn disable_raw_mode() -> Result<(), std::io::Error> {
    unsafe {
        if let Some(original) = ORIGINAL_TERMIOS {
            let result = tcsetattr(STDIN_FILENO, TCSANOW, &original);
            if result != 0 {
                return Err(std::io::Error::last_os_error());
            }
        }
        Ok(())
    }
}

pub fn enter_alternate_screen() -> Result<(), std::io::Error> {
    print!("\x1b[?1049h\x1b[?25l");
    io::stdout().flush()
}

pub fn leave_alternate_screen() -> Result<(), std::io::Error> {
    print!("\x1b[?25h\x1b[?1049l\x1b[0m");
    io::stdout().flush()
}

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn new() -> Result<Self, std::io::Error> {
        enable_raw_mode()?;
        enter_alternate_screen()?;
        Ok(TerminalGuard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = leave_alternate_screen();
        let _ = disable_raw_mode();
    }
}

pub fn poll_stdin(timeout: Duration) -> Result<bool, std::io::Error> {
    let mut pfd = pollfd {
        fd: STDIN_FILENO,
        events: POLLIN,
        revents: 0,
    };

    let timeout_ms = timeout.as_millis() as libc::c_int;

    unsafe {
        let ret = poll(&mut pfd, 1, timeout_ms);
        if ret < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(ret > 0 && (pfd.revents & POLLIN) != 0)
    }
}

pub fn read_stdin_char() -> Result<u8, std::io::Error> {
    let mut buffer = [0; 1];
    io::stdin().read_exact(&mut buffer)?;
    Ok(buffer[0])
}
