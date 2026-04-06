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
- `src/modes/renice.rs` - Renice mode logic
- `src/modes/kill.rs` - Kill mode logic
- `src/modes/pause.rs` - Pause mode logic
- `src/modes/help.rs` - Help mode logic
- `src/input.rs` - Keyboard input handling

### Debugging

- Use `eprintln!` for timing debug output (goes to stderr, doesn't interfere with UI)
- Add timing instrumentation to identify slow operations
- Example: `let start = Instant::now(); ... eprintln!("Operation: {:?}", start.elapsed());`

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