# Refactoring Opportunities

This document outlines structural and performance optimization opportunities for the `easyfetch` codebase.

---

## 1. TUI Coordinate Overflow Control (`src/view/app.rs`)

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
