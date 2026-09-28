use crate::config::key;
use crate::system::systemmonitor::{
    MetricLevel, MetricRow, MonitorType, fetch_all_data, initalize_monitors, update_monitors,
};
use crate::view::ascii::{
    AsciiMode, draw_ascii_box, draw_sys_ascii_box, resolve_ascii,
};
use crate::view::tui::{Color, Rect, ScreenBuffer, Status, TuiRow, draw_box};
use std::time::{Duration, Instant};

pub trait TuiApp {
    fn update(&mut self);
    fn draw(&mut self, buffer: &mut ScreenBuffer, area: Rect);
    fn handle_input(&mut self, key: u8) -> bool;
}

pub struct MonitorApp {
    pub monitors: Vec<MonitorType>,
    pub refresh_interval: Duration,
    pub last_refresh: Instant,
    pub config: crate::config::AppConfig,
}

impl MonitorApp {
    pub fn new(refresh_secs: u64) -> Self {
        let config = crate::config::AppConfig::load();
        let interval_secs = if refresh_secs != 1 {
            refresh_secs
        } else {
            config.refresh_interval.unwrap_or(refresh_secs)
        };
        MonitorApp {
            monitors: initalize_monitors(),
            refresh_interval: Duration::from_secs(interval_secs.max(1)),
            last_refresh: Instant::now(),
            config,
        }
    }
}

impl TuiApp for MonitorApp {
    fn update(&mut self) {
        if self.last_refresh.elapsed() >= self.refresh_interval {
            update_monitors(&mut self.monitors);
            self.last_refresh = Instant::now();
        }
    }

    fn draw(&mut self, buffer: &mut ScreenBuffer, area: Rect) {
        if area.width < 20 || area.height < 6 {
            return;
        }

        let work_area = Rect::new(
            2,
            1,
            area.width.saturating_sub(4),
            area.height.saturating_sub(2),
        );

        let term_h = area.height;
        let is_wide = work_area.width >= 90;

        let ascii_info = resolve_ascii(&self.config, is_wide, term_h);
        let ascii_mode = ascii_info.mode;
        let ascii_title = &ascii_info.title;
        let ascii_lines = &ascii_info.lines;
        let ascii_palette = &ascii_info.palette;

        let system_data = fetch_all_data(&mut self.monitors);

        let needed_ascii_h = match ascii_mode {
            AsciiMode::Full => ascii_lines.len() + 4,
            AsciiMode::Compact => ascii_lines.len() + 2,
            AsciiMode::None => 0,
        };

        let max_y = work_area.y + work_area.height;

        if is_wide {
            let left_width = (work_area.width / 2).saturating_sub(1);
            let right_x = work_area.x + left_width + 1;
            let right_width = work_area.width.saturating_sub(left_width + 1);

            let mut sys_item = None;
            let mut other_items = Vec::new();
            for item in system_data {
                if item.0.contains("System") && sys_item.is_none() {
                    sys_item = Some(item);
                } else {
                    other_items.push(item);
                }
            }

            let gap = if term_h < 26 { 0 } else { 1 };

            let left_start_y = if ascii_mode != AsciiMode::None {
                let ascii_h = needed_ascii_h.min(max_y.saturating_sub(work_area.y));
                let ascii_rect = Rect::new(work_area.x, work_area.y, left_width, ascii_h);
                let bottom_ascii_y = draw_ascii_box(
                    buffer,
                    ascii_rect,
                    ascii_title,
                    ascii_lines,
                    ascii_palette,
                );
                bottom_ascii_y + gap
            } else {
                work_area.y
            };

            let mut right_start_y = work_area.y;
            if let Some(sys) = sys_item {
                let sys_h = compute_needed_height(&sys.1).min(max_y.saturating_sub(right_start_y));
                let sys_rect = Rect::new(right_x, right_start_y, right_width, sys_h);
                let rows = convert_to_tui_rows(&sys.1);
                let bottom_sys_y = draw_box(buffer, sys_rect, &sys.0, &rows);
                right_start_y = bottom_sys_y + gap;
            }

            let mut left_items = Vec::new();
            let mut right_items = Vec::new();

            if term_h >= 24 {
                // Generous vertical room: Left gets Temperatures, Right gets Memory, Storage, Network
                for item in other_items {
                    if item.0.contains("Temperatures") {
                        left_items.push(item);
                    } else {
                        right_items.push(item);
                    }
                }
            } else {
                // Constrained vertical room (< 24 rows):
                // Sort by height descending so largest items fit first into the shorter column
                let mut sorted = other_items;
                sorted.sort_by_key(|it| std::cmp::Reverse(compute_needed_height(&it.1)));

                let mut left_h = left_start_y;
                let mut right_h = right_start_y;

                for item in sorted {
                    let h = compute_needed_height(&item.1) + gap;
                    if left_h <= right_h {
                        left_h += h;
                        left_items.push(item);
                    } else {
                        right_h += h;
                        right_items.push(item);
                    }
                }
            }

            draw_column(
                buffer,
                work_area.x,
                left_start_y,
                left_width,
                max_y,
                &left_items,
                gap,
            );

            draw_column(
                buffer,
                right_x,
                right_start_y,
                right_width,
                max_y,
                &right_items,
                gap,
            );
        } else {
            // 1-column layout
            let mut sys_item = None;
            let mut other_items = Vec::new();
            for item in system_data {
                if item.0.contains("System") && sys_item.is_none() {
                    sys_item = Some(item);
                } else {
                    other_items.push(item);
                }
            }

            let next_y = if ascii_mode == AsciiMode::Full {
                let ascii_h = needed_ascii_h.min(max_y.saturating_sub(work_area.y));
                let ascii_rect = Rect::new(work_area.x, work_area.y, work_area.width, ascii_h);
                let bottom_ascii_y = draw_ascii_box(
                    buffer,
                    ascii_rect,
                    ascii_title,
                    ascii_lines,
                    ascii_palette,
                );
                let mut cur = bottom_ascii_y + 1;
                if let Some(sys) = sys_item {
                    let sys_h = compute_needed_height(&sys.1).min(max_y.saturating_sub(cur));
                    let sys_rect = Rect::new(work_area.x, cur, work_area.width, sys_h);
                    let rows = convert_to_tui_rows(&sys.1);
                    cur = draw_box(buffer, sys_rect, &sys.0, &rows) + 1;
                }
                cur
            } else if ascii_mode == AsciiMode::Compact {
                // In compact 1-column mode, merge ASCII logo and System Info side-by-side!
                let rows = sys_item
                    .as_ref()
                    .map(|s| convert_to_tui_rows(&s.1))
                    .unwrap_or_default();
                let needed_h = (ascii_lines.len().max(rows.len()) + 2)
                    .min(max_y.saturating_sub(work_area.y));
                let box_rect = Rect::new(work_area.x, work_area.y, work_area.width, needed_h);
                let bottom_y = draw_sys_ascii_box(
                    buffer,
                    box_rect,
                    ascii_title,
                    ascii_lines,
                    ascii_palette,
                    &rows,
                );
                bottom_y + 1
            } else {
                // AsciiMode::None: minimal, just draw System Info box with distro badge
                let mut cur = work_area.y;
                if let Some(sys) = sys_item {
                    let sys_h = compute_needed_height(&sys.1).min(max_y.saturating_sub(cur));
                    let sys_rect = Rect::new(work_area.x, cur, work_area.width, sys_h);
                    let rows = convert_to_tui_rows(&sys.1);
                    cur = draw_box(buffer, sys_rect, &format!("{}── {}", ascii_title, sys.0.trim()), &rows) + 1;
                }
                cur
            };

            // Prioritize Memory, Storage, Network, then Temperatures in 1-column
            let mut ordered_items = Vec::new();
            let mut temp_item = None;
            for item in other_items {
                if item.0.contains("Temperatures") {
                    temp_item = Some(item);
                } else {
                    ordered_items.push(item);
                }
            }
            if let Some(t) = temp_item {
                ordered_items.push(t);
            }

            let gap = if term_h < 26 { 0 } else { 1 };
            draw_column(
                buffer,
                work_area.x,
                next_y,
                work_area.width,
                max_y,
                &ordered_items,
                gap,
            );
        }
    }

    fn handle_input(&mut self, typed_key: u8) -> bool {
        // Exit on 'q' or Ctrl+C (0x03)
        typed_key != key::QUIT && typed_key != 3
    }
}

fn compute_needed_height(box_data: &[MetricRow]) -> usize {
    box_data.len() + 2
}

fn component_color_for_label(label: &str) -> Option<Color> {
    if label.starts_with("CPU") {
        Some(Color::Blue)
    } else if label.starts_with("GPU") {
        Some(Color::Magenta)
    } else if label.starts_with("NVMe") {
        Some(Color::Green)
    } else if label.starts_with("Memory") {
        Some(Color::Yellow)
    } else if label.starts_with("Wi-Fi") {
        Some(Color::Cyan)
    } else if label.starts_with("Motherboard") {
        Some(Color::White)
    } else {
        None
    }
}

fn convert_to_tui_rows(box_data: &[MetricRow]) -> Vec<TuiRow> {
    box_data
        .iter()
        .map(|metric| {
            let status = match metric.level {
                MetricLevel::Ok => Status::Normal,
                MetricLevel::Warn => Status::Warning,
                MetricLevel::Crit => Status::Critical,
                MetricLevel::Unknown => Status::Unknown,
            };
            let mut row = TuiRow::new(metric.label.clone(), metric.value.clone(), status);
            if let Some(p) = metric.progress {
                row = row.with_progress(p);
            }
            if let Some(c) = component_color_for_label(&metric.label) {
                row = row.with_color(c);
            }
            row
        })
        .collect()
}

fn draw_column(
    buffer: &mut ScreenBuffer,
    col_x: usize,
    start_y: usize,
    col_width: usize,
    max_y: usize,
    items: &[(String, Vec<MetricRow>)],
    gap: usize,
) {
    let mut current_y = start_y;
    for (title, box_data) in items {
        if current_y >= max_y {
            break;
        }
        let needed_height = compute_needed_height(box_data);
        let available_height = max_y.saturating_sub(current_y);
        if available_height < 3 {
            break;
        }
        let box_height = needed_height.min(available_height);
        let box_rect = Rect::new(col_x, current_y, col_width, box_height);
        let tui_rows = convert_to_tui_rows(box_data);
        let bottom_y = draw_box(buffer, box_rect, title, &tui_rows);
        current_y = bottom_y + gap;
    }
}
