#![allow(dead_code)]

use std::io::{self, Write as IoWrite};
use std::fmt::Write;
use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};
use crate::view::colorize::Colorize;

pub enum Status {
    Normal,
    Warning,
    Critical,
    Unknown,
}

pub struct TuiRow {
    pub label: String,
    pub value: String,
    pub status: Status,
    pub progress: Option<f64>,
}

impl TuiRow {
    pub fn new(label: String, value: String, status: Status) -> Self {
        TuiRow {
            label,
            value,
            status,
            progress: None,
        }
    }

    pub fn simple(label: String, value: String) -> Self {
        TuiRow {
            label,
            value,
            status: Status::Normal,
            progress: None,
        }
    }

    pub fn with_progress(mut self, progress: f64) -> Self {
        self.progress = Some(progress);
        self
    }
}

pub fn clear_screen(buffer: &mut String) {
    write!(buffer, "\x1B[2J\x1B[H").unwrap();
}

pub fn move_cursor(buffer: &mut String, x: usize, y: usize) {
    write!(buffer, "\x1B[{};{}H", y, x).unwrap();
}

pub fn get_terminal_size() -> (usize, usize) {
    unsafe {
        let mut w: winsize = std::mem::zeroed();

        if ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut w) == 0 {
            (w.ws_col as usize, w.ws_row as usize)
        } else {
            (80, 24)
        }
    }
}

pub fn draw_dynamic_box(
    buffer: &mut String,
    start_x: usize, 
    start_y: usize, 
    width: usize,
    title: &str,
    items: &[TuiRow]
) -> usize {
    let inner_width = width.saturating_sub(2);

    move_cursor(buffer, start_x, start_y);
    write!(buffer, "{}", "╭".blue().bold()).unwrap();
    if !title.is_empty() && inner_width > title.chars().count() + 4 {
        let left_padding = 2;
        let right_padding = inner_width - title.chars().count() - left_padding;
        write!(
            buffer,
            "{}{}{}{}",
            "─".repeat(left_padding).blue().bold(),
            title.white().bold(),
            "─".repeat(right_padding).blue().bold(),
            "╮".blue().bold()
        ).unwrap();
    } else {
        let horizontal_line = "─".repeat(inner_width);
        write!(
            buffer,
            "{}{}",
            horizontal_line.as_str().blue().bold(),
            "╮".blue().bold()
        ).unwrap();
    }

    let mut current_y = start_y + 1;
    
    for item in items.iter() {
        move_cursor(buffer, start_x, current_y);
        
        let mut safe_value = item.value.clone();
        let mut visible_chars = item.label.chars().count() + 3 + safe_value.chars().count();
        
        if visible_chars > inner_width {
            let allowed_len = inner_width.saturating_sub(item.label.chars().count() + 6);

            if allowed_len > 0 {
                safe_value = safe_value.chars().take(allowed_len).collect::<String>() + "...";
            } else {
                safe_value = "".to_string();
            }
        }

        let final_visible = item.label.chars().count() + 3 + safe_value.chars().count();
        let padding_spaces = inner_width.saturating_sub(final_visible);
        let padding_string = " ".repeat(padding_spaces);

        let colored_value = match item.status {
            Status::Normal   =>  safe_value.as_str().cyan(),
            Status::Warning  =>  safe_value.as_str().yellow().bold(),
            Status::Critical =>  safe_value.as_str().red().bold(),
            Status::Unknown  =>  safe_value.as_str().dark_grey(),
        };
        
        write!(
            buffer,
            "{} {} {}{} {}",
            "│".blue().bold(),
            item.label.as_str().green(),
            colored_value,
            padding_string,
            "│".blue().bold()
        ).unwrap();

        current_y += 1;

        if let Some(percentage) = item.progress {
            move_cursor(buffer, start_x, current_y);
            let bar = draw_progress_bar(inner_width - 2, percentage);
            write!(buffer, "{} {} {}", "│".blue().bold(), bar, "│".blue().bold()).unwrap();

            current_y += 1;
        }
    }

    let horizontal_line = "─".repeat(inner_width);
    move_cursor(buffer, start_x, current_y);
    write!(buffer, "{}{}{}", "╰".blue().bold(), horizontal_line.as_str().blue().bold(), "╯".blue().bold()).unwrap();
    
    current_y + 1
}

pub fn draw_progress_bar(inner_width: usize, percentage: f64) -> String {
    let percentage_text = format!("{:02.0}%", percentage);
    let reserved_len = percentage_text.len() + 2;
    let bar_width = inner_width.saturating_sub(reserved_len);
    let fraction = percentage / 100.0; 
    let filled_length = (bar_width as f64 * fraction).round() as usize;
    let empty_length = bar_width.saturating_sub(filled_length);
    
    let actual_bar_len = filled_length + empty_length;
    let padding = " ".repeat(inner_width.saturating_sub(actual_bar_len + reserved_len));
    
    format!("{}{}  {}{}", "█".repeat(filled_length), "░".repeat(empty_length), percentage_text, padding)
}
