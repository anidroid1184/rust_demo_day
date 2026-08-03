# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Tags are aligned with the milestones in [`docs/MILESTONES.md`](docs/MILESTONES.md).

## [Unreleased]

## [v0.2.0] - 2026-08-03

### Milestone 2 — Interrupt-Driven RX

- Moved UART0 RX from blocking to interrupt-driven.
- Added an ISR that captures each incoming byte on `RxFifoFull` and stores it
  in a `critical_section::Mutex<RefCell<Option<u8>>>` static buffer.
- Main loop reads from the shared buffer and echoes the received byte back.
- Added a blinking blue LED (GPIO26) as a run indicator.
- Simplified the Wokwi simulation circuit to a single LED.
- Added module organization (`uart_handler`, `ring_buffer`, `command_parser`)
  and milestone documentation (EN + ES).

## [v0.1.0] - 2026-07-05

### Milestone 1 — UART Echo (Blocking)

- Initialized UART0 at 115200 baud on GPIO1 (TX) / GPIO3 (RX).
- Implemented blocking UART echo — any received byte is written straight back.
- Verified in Wokwi simulation.
- Set up project structure: crate scaffolding, module declarations, and
  documentation scaffolding.

[Unreleased]: https://github.com/anidroid1184/uart-command-rust/compare/v0.2.0...HEAD
[v0.2.0]: https://github.com/anidroid1184/uart-command-rust/compare/v0.1.0...v0.2.0
[v0.1.0]: https://github.com/anidroid1184/uart-command-rust/releases/tag/v0.1.0
