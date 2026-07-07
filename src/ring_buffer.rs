//! Lock-free byte transfer between ISR (producer) and main loop (consumer).
//! Uses bbqueue to avoid mutexes and critical sections.
// Phase 3: bbqueue producer/consumer setup will live here.
// Responsible for: lock-free byte transfer between ISR context and main loop context.
