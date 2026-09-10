use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc::channel, Arc, Mutex};
use std::thread;
use std::time::Duration;

const BLACKLIST: &[&str] = &[
    "target",
    ".git",
    ".cache",
    "venv",
    "__pycache__",
    "Steam",
    ".steam",
    ".vscode",
    "node_modules",
    ".cargo",
    ".rustup",
    ".local",
    ".config",
    "go",
    ".npm",
    ".mozilla",
    ".var",
    "Android",
];

pub fn search_directory(
    dir: &Path, 
    target: &str,
    use_blacklist: bool,
    cancelled: Arc<AtomicBool>,
) -> Vec<PathBuf> {
    // 1. Shared State
    let target_lower = Arc::new(target.to_lowercase());
    let work_queue = Arc::new(Mutex::new(vec![dir.to_path_buf()]));
    let active_tasks = Arc::new(AtomicUsize::new(1)); 

    // 2. The Communication Channel
    let (tx, rx) = channel();
    let mut thread_handles = vec![];

    let core_count = thread::available_parallelism()
                        .map(|n| n.get())
                        .unwrap_or(4);

    // 3. Spawn worker threads
    for _ in 0..core_count {
        let queue = Arc::clone(&work_queue);
        let tx = tx.clone();
        let active = Arc::clone(&active_tasks);
        let target = Arc::clone(&target_lower);
        let cancelled = Arc::clone(&cancelled);

        let handle = thread::spawn(move || {
            loop {
                if cancelled.load(Ordering::Relaxed) {
                    break;
                }
                // Safely grab a folder from the queue
                let current_dir = {
                    let mut q = queue.lock().unwrap();
                    q.pop()
                };

                if let Some(d) = current_dir {
                    // Optional: Uncomment this line if you ever want to see exactly what folders it is scanning!
                    // println!("Scanning: {:?}", d.display());

                    if let Ok(entries) = fs::read_dir(&d) {
                        for entry in entries.flatten() {
                            if cancelled.load(Ordering::Relaxed) {
                                break;
                            }
                            let path = entry.path();
                            
                            if path.is_dir() {
                                // Extract the folder name safely to check our Blacklist
                                if let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) {
                                    // THE BLACKLIST: Instantly skip these massive/useless directories
                                    if use_blacklist && BLACKLIST.contains(&folder_name) {
                                        continue;
                                    }
                                }
                                
                                // Safe to search: Add to queue and increment counter
                                queue.lock().unwrap().push(path);
                                active.fetch_add(1, Ordering::SeqCst);
                            } 
                            else if path.file_name()
                                .and_then(|n| n.to_str())
                                .is_some_and(|n| n.to_lowercase().contains(target.as_str())) 
                            {
                                // File match found! Transmit back to main thread
                                let _ = tx.send(path);
                            }
                        }
                    }
                    // Finished processing this folder
                    active.fetch_sub(1, Ordering::SeqCst);
                } else {
                    // If queue is empty, check if all threads are finished
                    if active.load(Ordering::SeqCst) == 0 {
                        break; 
                    }
                    // Wait a tiny fraction of a millisecond for other threads to find more folders
                    thread::sleep(Duration::from_millis(1));
                }
            }
        });
        thread_handles.push(handle);
    }

    // 4. Drop the transmitter so the receiver can finish
    drop(tx);

    // 5. Collect all transmitted results
    let mut matching_results = Vec::new();
    for result in rx {
        matching_results.push(result);
    }

    // 6. Ensure all threads shut down cleanly
    for handle in thread_handles {
        handle.join().unwrap();
    }

    matching_results
}
