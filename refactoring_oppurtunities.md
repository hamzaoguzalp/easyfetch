# Refactoring Opportunities

This document outlines structural and performance optimization opportunities for the `easyfetch` codebase.

---

## 1. File Search Polling (`src/service/filesearch.rs`)

### Current Implementation
When a worker thread finds the directory queue is empty, it sleeps for a short duration to wait for other threads to discover and push more subdirectories:
```rust
// filesearch.rs:89-91
if active.load(Ordering::SeqCst) == 0 {
    break; 
}
thread::sleep(Duration::from_millis(1));
```

### The Issue
This is a **spin-polling loop**. It introduces:
* **Arbitrary Latency**: A thread waits for exactly 1ms even if work becomes available in 5 microseconds.
* **CPU Overhead**: Threads wake up repeatedly just to check the queue state.

### Suggested Fixes
1. **Condition Variable (`std::sync::Condvar`)**:
   Use a `Mutex` along with a `Condvar` to put threads to sleep when the queue is empty. Waking threads instantly using `condvar.notify_all()` or `condvar.notify_one()` when a thread pushes new directories.
2. **Rayon / Parallel Iterator**:
   If directory searching is kept, replace custom channel/queue thread management with a parallel filesystem walker library or `rayon`'s work-stealing threadpool for parallel iteration.

---

## 2. TUI Coordinate Overflow Control (`src/main.rs`)

### Current Implementation
Boxes are stacked vertically with increments of height and padding:
```rust
current_y = bottom_y + padding_between;
```

### The Issue
If the sum of box heights exceeds the terminal rows (e.g., drawing 45 lines of boxes in a 24-row window):
1. Drawing to row indices past the terminal height forces the terminal to automatically scroll the content up.
2. The viewport coordinate system stays relative to the top of the visible screen.
3. Subsequent absolute cursor movements (`\x1B[y;xH`) targeting old row indices will overwrite the wrong lines instead of printing at the bottom.

### Suggested Fixes
1. **Collapsible Sections**:
   Allow folders/boxes to be collapsed/expanded via keyboard controls (e.g., up/down arrows to select a box, Enter to toggle visibility).
2. **Pagination / Scrolling Viewport**:
   Add a scrolling offset. Adjust `start_y` of each box based on scroll offsets and clear/render only the visible portion of the TUI.
3. **Height Clamping**:
   Stop rendering boxes once the terminal height boundary is reached to prevent layout corruption.

agy --conversation=ca03dbdb-0cc2-4485-8723-3ace920d0c14
