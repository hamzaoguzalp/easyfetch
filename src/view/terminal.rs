use libc::{termios, tcgetattr, tcsetattr, TCSANOW, STDIN_FILENO, ECHO, ICANON};
use libc::{poll, pollfd, POLLIN};
use std::process::Command;
use std::path::Path;
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

pub fn open_in_editor(file_path: &Path) -> Result<(), std::io::Error> {
    disable_raw_mode()?;
    print!("\x1b[2J\x1b[H");
    let _ = io::stdout().flush();

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
    let _status = Command::new(editor)
        .arg(file_path)
        .status()?;
        
    enable_raw_mode()?;
    Ok(())
}
