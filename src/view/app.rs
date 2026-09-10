#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc};
use crate::system::systemmonitor::{MonitorType, MetricLevel, initalize_monitors, update_monitors, fetch_all_data};
use crate::view::tui::{draw_dynamic_box, Status, TuiRow};
use crate::service::filesearch::search_directory;
use crate::view::terminal::open_in_editor;
use crate::config::{key, layout};

pub trait TuiApp {
    fn update(&mut self);
    fn draw(&mut self, buffer: &mut String, width: usize);
    fn handle_input(&mut self, key: u8) -> bool;
}

pub struct MonitorApp {
    pub monitors: Vec<MonitorType>,
}

impl MonitorApp {
    pub fn new() -> Self {
        MonitorApp {
            monitors: initalize_monitors(),
        }
    }
}

impl TuiApp for MonitorApp {
    fn update(&mut self) {
        update_monitors(&mut self.monitors);
    }

    fn draw(&mut self, buffer: &mut String, width: usize) {
        let margin = 2;
        let padding_between = 1;
        let mut current_y = 2;

        let system_data = fetch_all_data(&mut self.monitors);
        for (title, box_data) in system_data {
            let tui_rows: Vec<TuiRow> = box_data
                .into_iter()
                .map(|metric| {
                    let status = match metric.level {
                        MetricLevel::Ok       =>  Status::Normal,
                        MetricLevel::Warn     =>  Status::Warning,
                        MetricLevel::Crit     =>  Status::Critical,
                        MetricLevel::Unknown  =>  Status::Unknown,
                    };
                    let mut row = TuiRow::new(metric.label, metric.value, status);
                    if let Some(p) = metric.progress {
                        row = row.with_progress(p);
                    }
                    row
                })
                .collect();

            let bottom_y = draw_dynamic_box(
                buffer, 
                margin, 
                current_y, 
                width, 
                &title, 
                &tui_rows
            );
            current_y = bottom_y + padding_between;
        }
    }

    fn handle_input(&mut self, typed_key: u8) -> bool {
        match typed_key {
            key::QUIT => false,
            _ => true,
        }
    }
}

// ==========================================
// 2. FILE SEARCH VIEW
// ==========================================
pub struct SearchApp {
    pub search_query: String,
    pub search_results: Vec<PathBuf>,
    pub selected_idx: usize,
    pub search_dir: PathBuf,
    
    // Background search threading
    pub search_cancelled: Arc<AtomicBool>,
    pub search_rx: Option<mpsc::Receiver<Vec<PathBuf>>>,

    // Modal typing toggle
    pub typing: bool,
}

impl SearchApp {
    pub fn new(search_dir: PathBuf, initial_query: String) -> Self {
        let mut app = SearchApp {
            search_query: initial_query,
            search_results: Vec::new(),
            selected_idx: 0,
            search_dir,
            search_cancelled: Arc::new(AtomicBool::new(false)),
            search_rx: None,
            typing: false,
        };
        if !app.search_query.is_empty() {
            app.run_search();
        }
        app
    }

    pub fn run_search(&mut self) {
        // Cancel the current active search thread
        self.search_cancelled.store(true, Ordering::Relaxed);
        
        if self.search_query.is_empty() {
            self.search_results.clear();
            self.selected_idx = 0;
            self.search_rx = None;
            return;
        }

        // Initialize new cancellation and channel state
        self.search_cancelled = std::sync::Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        self.search_rx = Some(rx);

        let target = self.search_query.clone();
        let dir = self.search_dir.clone();
        let cancelled = Arc::clone(&self.search_cancelled);

        // Spawn background thread to prevent TUI blocking/freezing
        std::thread::spawn(move || {
            let results = search_directory(&dir, &target, true, cancelled);
            let _ = tx.send(results);
        });
    }
}

impl TuiApp for SearchApp {
    fn update(&mut self) {
        // Poll background search receiver channel for new results
        if let Some(ref rx) = self.search_rx
        && let Ok(results) = rx.try_recv() {
            self.search_results = results;
            self.selected_idx = 0;
            self.search_rx = None; // Reset receiver
        }
    }

    fn draw(&mut self, buffer: &mut String, width: usize) {
        let margin = 2;
        let padding_between = 1;
        let mut current_y = 2;

        // 1. Draw input bar
        let display_value = self.search_query.clone();
        let query_item = vec![TuiRow::simple(
            "Query :".to_string(),
            display_value,
        )];

        let input_title = if self.typing {
            " File Search [ TYPING MODE - Press Enter to finish ] "
        } else {
            " File Search [ NAVIGATING - Press 's' to edit query, 'q' to quit ] "
        };

        let bottom_y = draw_dynamic_box(
            buffer, 
            margin, 
            current_y, 
            width, 
            input_title, 
            &query_item
        );
        current_y = bottom_y + padding_between;

        // 2. Draw results list (capped at 10 results for page display)
        let mut results_items = Vec::new();
        for (idx, path) in self.search_results.iter().take(10).enumerate() {
            let prefix = if idx == self.selected_idx { "> " } else { "  " };
            let label = format!("{}{}", prefix, path.display());
            results_items.push(TuiRow::new(
                label,
                "".to_string(),
                if idx == self.selected_idx { Status::Warning } else { Status::Normal },
            ));
        }

        if results_items.is_empty() {
            if self.search_rx.is_some() {
                results_items.push(TuiRow::new(
                    "Searching...".to_string(),
                    "".to_string(),
                    Status::Warning,
                ));
            } else {
                results_items.push(TuiRow::new(
                    "No matching files found.".to_string(),
                    "".to_string(),
                    Status::Unknown,
                ));
            }
        }

        draw_dynamic_box(
            buffer, 
            margin, 
            current_y, 
            width, 
            " Search Results (j/k: Navigate, Enter: Open in Editor) ", 
            &results_items
        );
    }

    fn handle_input(&mut self, typed_key: u8) -> bool {
        if self.typing {
            match typed_key {
                // Enter finishes typing and exits typing mode
                key::ENTER | key::ENTER_ALTERNATIVE => {
                    self.typing = false;
                    self.run_search();
                }
                // Backspace (ASCII 127 and 8)
                key::BACKSPACE | key::BACKSPACE_ALTERNATIVE => {
                    self.search_query.pop();
                    self.run_search();
                }
                // Text input
                typed_key if typed_key.is_ascii_graphic() || typed_key == key::SPACE => {
                    self.search_query.push(typed_key as char);
                    self.run_search();
                }
                _ => {}
            }
            true
        } else {
            match typed_key {
                // Quit application
                key::QUIT => {
                    self.search_cancelled.store(true, Ordering::Relaxed);
                    false
                }
                // Enter typing mode
                key::EDIT_QUERY => {
                    self.typing = true;
                    true
                }
                // Vim navigation keys
                key::NAVIGATE_DOWN => {
                    if !self.search_results.is_empty() {
                        let max_items = self.search_results.len().min(10);
                        self.selected_idx = (self.selected_idx + 1) % max_items;
                    }
                    true
                }
                key::NAVIGATE_UP => {
                    if !self.search_results.is_empty() {
                        let max_items = self.search_results.len().min(10);
                        if self.selected_idx == 0 {
                            self.selected_idx = max_items - 1;
                        } else {
                            self.selected_idx -= 1;
                        }
                    }
                    true
                }
                // Enter opens selected file in editor
                key::ENTER | key::ENTER_ALTERNATIVE => {
                    if !self.search_results.is_empty() {
                        let path = &self.search_results[self.selected_idx];
                        let _ = open_in_editor(path);
                    }
                    true
                }
                _ => true,
            }
        }
    }
}
