# RTOP Development Guidelines

## OOP Requirements

1. One class per file
2. Each method should do one thing only
3. Classes should be small and focused
4. Makes debugging easier - isolate issues to specific classes

## Code Organization

The application should be refactored to use OOP principles with separate files:

- `src/main.rs` - Entry point, initializes and runs the app
- `src/system_monitor.rs` - System stats collection (CPU, RAM, SWAP, load)
- `src/process_list.rs` - Process listing with nice values
- `src/zram_stats.rs` - ZRAM statistics
- `src/ui.rs` - Terminal UI rendering
- `src/overview.rs` - System overview/header rendering
- `src/color.rs` - Color scheme and ANSI codes
- `src/process_table_render.rs` - Process table rendering using tabled crate
- `src/modes/renice.rs` - Renice mode logic
- `src/modes/kill.rs` - Kill mode logic
- `src/modes/pause.rs` - Pause mode logic
- `src/modes/help.rs` - Help mode logic
- `src/input.rs` - Keyboard input handling

### Architecture Rules

1. **All UI must go through ui.rs** - No direct printing from main.rs or other modules
2. UI-related logic (headers, footers, help, process tables) should be in ui.rs or called via ui.rs
3. **Always log to debug.log** - Use Logger, no eprintln! for debug output
4. **Every function must have at least one DEBUG log** - Log entry/exit or key operations

### Color Scheme

All colors are defined in `ui.rs` using two structs:

**Colors** - ANSI escape codes only:
- RED, GREEN, YELLOW, BLUE, CYAN, WHITE, BOLD, RESET

**ColorScheme** - Global thresholds (use `ColorScheme::global()`):
```rust
pub struct ColorScheme {
    pub health_excellent: i32,  // 85 - GREEN
    pub health_good: i32,       // 70 - CYAN
    pub health_ok: i32,         // 50 - YELLOW
    pub zram_excellent: f64,    // 3.0 - GREEN
    pub zram_good: f64,         // 2.0 - YELLOW
    pub load_high: f64,        // 1.5x cores - RED
    pub load_medium: f64,       // 1.0x cores - YELLOW
    pub cpu_high: f32,          // 80% - RED
    pub swap_high: f32,        // 50% - RED
}
```

**Usage:**
```rust
let colors = ColorScheme::global();
let health_color = colors.color_for_health(score);
let zram_color = colors.color_for_zram(ratio);
let load_color = colors.color_for_load(load, cores);
let cpu_color = colors.color_for_percent(cpu, threshold);
```

**Color values:** GREEN ≥ excellent, CYAN ≥ good, YELLOW ≥ ok, RED otherwise

### Debugging

- Add timing instrumentation to identify slow operations
- Example: `let start = Instant::now(); ... logger.log_timed("Operation", start);`

### Requirements

1. Renice/kill modes must have completely frozen process lists that never update until mode exits
2. Key presses must be instant with no delay
3. Process list should include NI (nice) column after PID

### Dependencies

- sysinfo - system information
- libc - system calls
- tabled - table rendering (with ANSI color support)
- clap - CLI arguments

### Testing

- Add unit tests for each class/method where appropriate
- Test core functionality like process sorting, health calculations, nice value handling
- Use Rust's built-in `#[test]` attribute
- Run tests with `cargo test`
- **Coverage target: >80%** - Ensure most modules have tests