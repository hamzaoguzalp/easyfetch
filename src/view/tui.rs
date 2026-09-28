#![allow(dead_code)]

use libc::{STDOUT_FILENO, TIOCGWINSZ, ioctl, winsize};
use std::fmt::Write as FmtWrite;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub color: Option<Color>,
}

impl TuiRow {
    pub fn new(label: String, value: String, status: Status) -> Self {
        TuiRow {
            label,
            value,
            status,
            progress: None,
            color: None,
        }
    }

    pub fn simple(label: String, value: String) -> Self {
        TuiRow {
            label,
            value,
            status: Status::Normal,
            progress: None,
            color: None,
        }
    }

    pub fn with_progress(mut self, progress: f64) -> Self {
        self.progress = Some(progress);
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

pub fn get_terminal_size() -> (usize, usize) {
    unsafe {
        let mut w: winsize = std::mem::zeroed();
        if ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut w) == 0 && w.ws_col > 0 && w.ws_row > 0 {
            (w.ws_col as usize, w.ws_row as usize)
        } else {
            (80, 24)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Color {
    #[default]
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    DarkGrey,
}

impl Color {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "reset" => Some(Color::Reset),
            "black" => Some(Color::Black),
            "red" => Some(Color::Red),
            "green" => Some(Color::Green),
            "yellow" => Some(Color::Yellow),
            "blue" => Some(Color::Blue),
            "magenta" | "purple" => Some(Color::Magenta),
            "cyan" => Some(Color::Cyan),
            "white" => Some(Color::White),
            "grey" | "gray" | "darkgrey" | "darkgray" | "dark_grey" | "dark_gray" => {
                Some(Color::DarkGrey)
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Style {
    pub fg: Color,
    pub bold: bool,
}

impl Style {
    pub const fn reset() -> Self {
        Style {
            fg: Color::Reset,
            bold: false,
        }
    }

    pub const fn fg(fg: Color) -> Self {
        Style { fg, bold: false }
    }

    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            ch: ' ',
            style: Style::reset(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Rect {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }
}

pub struct ScreenBuffer {
    pub width: usize,
    pub height: usize,
    cells: Vec<Cell>,
}

impl ScreenBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width.saturating_mul(height);
        ScreenBuffer {
            width,
            height,
            cells: vec![Cell::default(); size],
        }
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        if self.width != new_width || self.height != new_height {
            self.width = new_width;
            self.height = new_height;
            self.cells = vec![Cell::default(); new_width.saturating_mul(new_height)];
        }
    }

    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::default();
        }
    }

    pub fn put_char(&mut self, x: usize, y: usize, ch: char, style: Style) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            if idx < self.cells.len() {
                self.cells[idx] = Cell { ch, style };
            }
        }
    }

    pub fn put_str(
        &mut self,
        x: usize,
        y: usize,
        text: &str,
        style: Style,
        max_width: usize,
    ) -> usize {
        let mut drawn = 0;
        for ch in text.chars() {
            if drawn >= max_width || x + drawn >= self.width {
                break;
            }
            self.put_char(x + drawn, y, ch, style);
            drawn += 1;
        }
        drawn
    }

    pub fn render(&self, out: &mut String) {
        out.clear();
        out.push_str("\x1b[H"); // Cursor to top-left
        let mut current_style = Style::reset();

        for y in 0..self.height {
            let _ = write!(out, "\x1b[{};1H", y + 1);

            for x in 0..self.width {
                let cell = self.cells[y * self.width + x];
                if cell.style != current_style {
                    emit_style_transition(out, cell.style);
                    current_style = cell.style;
                }
                out.push(cell.ch);
            }
        }

        if current_style != Style::reset() {
            out.push_str("\x1b[0m");
        }
    }
}

fn emit_style_transition(out: &mut String, to: Style) {
    out.push_str("\x1b[0m");
    if to.bold {
        out.push_str("\x1b[1m");
    }
    match to.fg {
        Color::Reset => {}
        Color::Black => out.push_str("\x1b[30m"),
        Color::Red => out.push_str("\x1b[31m"),
        Color::Green => out.push_str("\x1b[32m"),
        Color::Yellow => out.push_str("\x1b[33m"),
        Color::Blue => out.push_str("\x1b[34m"),
        Color::Magenta => out.push_str("\x1b[35m"),
        Color::Cyan => out.push_str("\x1b[36m"),
        Color::White => out.push_str("\x1b[37m"),
        Color::DarkGrey => out.push_str("\x1b[90m"),
    }
}

pub fn draw_box(buffer: &mut ScreenBuffer, area: Rect, title: &str, items: &[TuiRow]) -> usize {
    if area.width < 4 || area.height < 3 {
        return area.y;
    }

    let border_style = Style::fg(Color::DarkGrey);
    let title_style = Style::fg(Color::Cyan).bold();
    let label_style = Style::fg(Color::Green);

    let inner_width = area.width.saturating_sub(2);
    let start_x = area.x;
    let start_y = area.y;

    // Top border: ╭── Title ────╮
    buffer.put_char(start_x, start_y, '╭', border_style);

    let title_len = title.chars().count();
    if !title.is_empty() && inner_width > title_len + 4 {
        let left_pad = 2;
        for i in 0..left_pad {
            buffer.put_char(start_x + 1 + i, start_y, '─', border_style);
        }
        buffer.put_str(
            start_x + 1 + left_pad,
            start_y,
            title,
            title_style,
            inner_width.saturating_sub(left_pad),
        );
        let right_start = 1 + left_pad + title_len;
        for x in right_start..area.width - 1 {
            buffer.put_char(start_x + x, start_y, '─', border_style);
        }
    } else {
        for x in 1..area.width - 1 {
            buffer.put_char(start_x + x, start_y, '─', border_style);
        }
    }
    buffer.put_char(start_x + area.width - 1, start_y, '╮', border_style);

    let max_y = area.y + area.height - 1; // Reserved for bottom border
    let mut current_y = start_y + 1;

    let content_x = start_x + 2;
    let max_content_width = inner_width.saturating_sub(2);

    let max_label_len = items
        .iter()
        .map(|it| it.label.chars().count())
        .max()
        .unwrap_or(0);

    let has_any_progress = items.iter().any(|it| it.progress.is_some());

    // Uniform progress bar geometry for boxes containing progress bars
    let (bar_x, bar_width, is_compact_gauge) = if has_any_progress {
        let max_val_len = items
            .iter()
            .map(|it| it.value.chars().count())
            .max()
            .unwrap_or(0);

        let is_compact = items.iter().all(|it| it.value.chars().count() <= 6);
        let b_x = content_x + max_label_len + 2;

        if is_compact {
            // Compact gauge box (Temperatures): cap bar width at 16 chars, place value directly beside the bar!
            let b_w = 16.min(max_content_width.saturating_sub(max_label_len + max_val_len + 4));
            (b_x, b_w, true)
        } else {
            // Memory & Storage: dynamic expanding bar, right-aligned values
            let val_col_x = content_x + max_content_width.saturating_sub(max_val_len);
            let b_w = if val_col_x > b_x + 6 {
                val_col_x - b_x - 2
            } else {
                0
            };
            (b_x, b_w, false)
        }
    } else {
        (0, 0, false)
    };

    for item in items {
        if current_y >= max_y {
            break;
        }

        buffer.put_char(start_x, current_y, '│', border_style);

        let val_style = match item.status {
            Status::Normal => Style::fg(Color::Cyan),
            Status::Warning => Style::fg(Color::Yellow).bold(),
            Status::Critical => Style::fg(Color::Red).bold(),
            Status::Unknown => Style::fg(Color::DarkGrey),
        };

        let label_len = item.label.chars().count();
        if item.value.is_empty() {
            if label_len > max_content_width {
                let keep = max_content_width.saturating_sub(3);
                let truncated: String = item.label.chars().take(keep).collect();
                buffer.put_str(content_x, current_y, &truncated, label_style, keep);
                buffer.put_str(content_x + keep, current_y, "...", label_style, 3);
            } else {
                buffer.put_str(
                    content_x,
                    current_y,
                    &item.label,
                    label_style,
                    max_content_width,
                );
            }
        } else if has_any_progress {
            let val_len = item.value.chars().count();
            let total_len = label_len + 1 + val_len;

            if total_len > max_content_width {
                let allowed_val = max_content_width.saturating_sub(label_len + 4);
                buffer.put_str(content_x, current_y, &item.label, label_style, label_len);
                buffer.put_char(content_x + label_len, current_y, ' ', Style::reset());

                if allowed_val > 0 {
                    let truncated: String = item.value.chars().take(allowed_val).collect();
                    buffer.put_str(
                        content_x + label_len + 1,
                        current_y,
                        &truncated,
                        val_style,
                        allowed_val,
                    );
                    buffer.put_str(
                        content_x + label_len + 1 + allowed_val,
                        current_y,
                        "...",
                        val_style,
                        3,
                    );
                }
            } else {
                let row_label_style = if let Some(c) = item.color {
                    Style::fg(c)
                } else {
                    label_style
                };
                buffer.put_str(content_x, current_y, &item.label, row_label_style, label_len);

                let val_x = if is_compact_gauge {
                    bar_x + bar_width + 2
                } else {
                    content_x + max_content_width.saturating_sub(val_len)
                };
                buffer.put_str(val_x, current_y, &item.value, val_style, val_len);

                // Smooth inline sub-block progress bar
                if let Some(percentage) = item.progress
                    && bar_width >= 4
                {
                    draw_inline_bar(
                        buffer,
                        bar_x,
                        current_y,
                        bar_width,
                        percentage,
                        item.color,
                    );
                }
            }
        } else {
            // Text key-value row (Fastfetch style: values left-aligned close to labels)
            let val_len = item.value.chars().count();
            let val_col_x = content_x + max_label_len + 3;

            buffer.put_str(content_x, current_y, &item.label, label_style, label_len);

            if val_col_x + val_len <= content_x + max_content_width {
                buffer.put_str(val_col_x, current_y, &item.value, val_style, val_len);
            } else if content_x + label_len + 1 + val_len <= content_x + max_content_width {
                buffer.put_str(content_x + label_len + 1, current_y, &item.value, val_style, val_len);
            } else {
                let allowed = max_content_width.saturating_sub(label_len + 4);
                if allowed > 0 {
                    let truncated: String = item.value.chars().take(allowed).collect();
                    buffer.put_str(content_x + label_len + 1, current_y, &truncated, val_style, allowed);
                    buffer.put_str(content_x + label_len + 1 + allowed, current_y, "...", val_style, 3);
                }
            }
        }

        buffer.put_char(start_x + area.width - 1, current_y, '│', border_style);
        current_y += 1;
    }

    // Bottom border: ╰────────────╯
    buffer.put_char(start_x, current_y, '╰', border_style);
    for x in 1..area.width - 1 {
        buffer.put_char(start_x + x, current_y, '─', border_style);
    }
    buffer.put_char(start_x + area.width - 1, current_y, '╯', border_style);

    current_y + 1
}

fn draw_inline_bar(
    buffer: &mut ScreenBuffer,
    x: usize,
    y: usize,
    bar_width: usize,
    percentage: f64,
    color_override: Option<Color>,
) {
    if bar_width < 4 {
        return;
    }
    const SUB: [char; 8] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉'];
    let inner_len = bar_width.saturating_sub(2);
    let clamped = percentage.clamp(0.0, 100.0);

    let filled_style = if clamped >= 85.0 {
        Style::fg(Color::Red).bold()
    } else if clamped >= 65.0 {
        Style::fg(Color::Yellow).bold()
    } else if let Some(c) = color_override {
        Style::fg(c).bold()
    } else {
        Style::fg(Color::Cyan)
    };
    let empty_style = Style::fg(Color::DarkGrey);
    let bracket_style = Style::fg(Color::DarkGrey);

    buffer.put_char(x, y, '[', bracket_style);

    let total_eighths = ((clamped / 100.0) * (inner_len * 8) as f64).round() as usize;
    let full_blocks = (total_eighths / 8).min(inner_len);
    let remainder = total_eighths % 8;

    for i in 0..full_blocks {
        buffer.put_char(x + 1 + i, y, '█', filled_style);
    }

    if full_blocks < inner_len {
        if remainder > 0 {
            buffer.put_char(x + 1 + full_blocks, y, SUB[remainder], filled_style);
        } else {
            buffer.put_char(x + 1 + full_blocks, y, '░', empty_style);
        }
        for i in (full_blocks + 1)..inner_len {
            buffer.put_char(x + 1 + i, y, '░', empty_style);
        }
    }

    buffer.put_char(x + 1 + inner_len, y, ']', bracket_style);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_box_fastfetch_alignment() {
        let mut buf = ScreenBuffer::new(40, 10);
        let items = vec![
            TuiRow::new("OS".to_string(), "Arch Linux".to_string(), Status::Normal),
            TuiRow::new("Kernel".to_string(), "6.1.0".to_string(), Status::Normal),
        ];
        draw_box(&mut buf, Rect::new(0, 0, 40, 6), "System", &items);
        // Verify buffer drew correctly
        assert_eq!(buf.cells[0].ch, '╭');
    }

    #[test]
    fn test_draw_box_progress_bar_expansion() {
        let mut buf = ScreenBuffer::new(60, 10);
        let items = vec![
            TuiRow::new("RAM".to_string(), "4.0 GB (27%)".to_string(), Status::Normal).with_progress(27.0),
        ];
        draw_box(&mut buf, Rect::new(0, 0, 60, 6), "Memory", &items);
        assert_eq!(buf.cells[0].ch, '╭');
    }
}

