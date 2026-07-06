# rust-demo-day

Real-time UART CLI for ESP32, built in Rust (bare-metal, `no_std`).

A minimal interactive command-line interface that runs on an ESP32
microcontroller, receiving commands via serial and responding in real time. Built
with the Espressif Rust ecosystem (`esp-hal`, `espup`, Wokwi simulation).

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Project Structure](#project-structure)
- [Usage](#usage)
- [Troubleshooting](#troubleshooting)
- [Milestones](#milestones)
- [Hardware Diagram](#hardware-diagram)
- [Team](#team)

## Prerequisites

### Toolchain (esp)

The project targets `xtensa-esp32-none-elf`, Espressif's Xtensa architecture.
The standard Rust toolchain does **not** include this target.

**Check if the `esp` toolchain is installed:**

```bash
rustup toolchain list | grep esp
```

Expected output includes a line with `esp`.

**Verify the target is available:**

```bash
rustup run esp rustc --print target-list | grep xtensa-esp32-none-elf
```

Expected output: `xtensa-esp32-none-elf`

The toolchain is managed by [`espup`](https://github.com/esp-rs/espup), not by
rustup's standard channel system.

**Rust version:**

| Key | Value |
|-----|-------|
| Channel | `esp` (Espressif Rust fork, nightly-based) |
| Rustc | ~1.95.0-nightly |
| Target | `xtensa-esp32-none-elf` |
| Defined in | `rust-toolchain.toml` |

### Espressif cross-compilation tools

The Xtensa linker and LLVM/Clang for ESP32 are installed alongside the `esp`
toolchain. Verify:

```bash
# Xtensa GCC linker should be on PATH after sourcing export-esp.sh
xtensa-esp32-elf-ld --version
```

### Mise (optional)

This project can use [mise](https://mise.jdx.dev) for environment management,
but it is **not required**. The `esp` toolchain is installed and managed by
`espup`, not by mise.

If you use mise:
- The project `mise.toml` lists additional local tooling (if any).
- Make sure mise does **not** override `RUSTUP_TOOLCHAIN` — the project relies
  on `rust-toolchain.toml` to select the correct channel.

## Installation

### 1. Install espup

```bash
cargo install espup
```

### 2. Install the Espressif Rust toolchain

```bash
espup install -t esp32
```

> The `-t esp32` flag limits the installation to the ESP32 target (faster than
> `all`). ESP32 classic (Xtensa LX6) is the chip used in this project.

### 3. Set up environment variables

Every new terminal needs the Espressif tools on `PATH`:

```bash
source $HOME/export-esp.sh
```

Alternatively, add this line to your shell rc file (e.g. `~/.zshrc`):

```bash
alias get_esprs='. $HOME/export-esp.sh'
```

### 4. Verify the setup

```bash
# Active toolchain should be esp
rustup show active-toolchain

# Build should succeed
cargo build
```

### 5. Wokwi simulation (VS Code)

1. Install the "Wokwi for VS Code" extension.
2. Open the project in VS Code.
3. Press `F1` → `Wokwi: Start Simulator`.

The simulator uses `wokwi.toml` to locate the firmware binary and
`diagram.json` to define the hardware layout.

## Project Structure

```
rust-demo-day/
├── .cargo/
│   └── config.toml          # Target: xtensa-esp32-none-elf, build-std, rustflags
├── src/
│   ├── bin/
│   │   └── main.rs          # Entry point and main loop
│   ├── lib.rs               # Module declarations
│   ├── uart_handler.rs      # UART0 configuration and (future) ISR registration
│   ├── ring_buffer.rs       # (pending) Lock-free bbqueue producer/consumer
│   └── command_parser.rs    # (pending) Line accumulator and command dispatch
├── docs/
│   ├── MILESTONES.md        # Project milestones (English)
│   └── MILESTONES_ES.md     # Project milestones (Spanish)
├── diagram.json             # Wokwi hardware simulation layout
├── wokwi.toml               # Wokwi firmware path configuration
├── build.rs                 # Linker script for esp-hal
├── rust-toolchain.toml      # Toolchain channel: esp
├── Cargo.toml               # Dependencies (esp-hal, bbqueue, heapless, etc.)
├── plan.md                  # Original project plan and architecture notes
└── mise.toml                # Optional mise configuration (commented by default)
```

## Usage

### Build

```bash
cargo build
```

The binary is written to
`target/xtensa-esp32-none-elf/debug/rust-demo-day`.

### Flash and monitor (physical board)

```bash
espflash flash --monitor --chip esp32 target/xtensa-esp32-none-elf/debug/rust-demo-day
```

### Simulate (Wokwi)

1. Open the project in VS Code.
2. Press `F1` → `Wokwi: Start Simulator`.
3. The serial monitor opens automatically — type characters and observe
   responses.

## Troubleshooting

### `error: the option \`Z\` is only accepted on the nightly compiler`

**Cause:** The active toolchain is a stable release, but `.cargo/config.toml`
uses nightly-only flags (`-Z`).

**Fix:** Make sure the `esp` toolchain is active:

```bash
# Check which toolchain is active
rustup show active-toolchain

# The output should show 'esp', not a stable version like '1.80.1'
# If it shows a stable version, unset the overridden variable or check mise
unset RUSTUP_TOOLCHAIN
```

The project's `rust-toolchain.toml` sets `channel = "esp"`, which rustup
respects unless overridden by `RUSTUP_TOOLCHAIN` (from mise, a shell rc file,
or similar).

### `error: target 'xtensa-esp32-none-elf' not installed`

**Cause:** The `esp` toolchain is missing the Xtensa target.

**Fix:** Re-run `espup` with the ESP32 target:

```bash
espup update -t esp32
source $HOME/export-esp.sh
```

### Wokwi does not start or shows a blank screen

1. Verify `wokwi.toml` points to the correct ELF path:
   `target/xtensa-esp32-none-elf/debug/rust-demo-day`
2. Run `cargo build` first so the binary exists.
3. Check `diagram.json` for correct pin assignments.

### LED does not light up in simulation

The current code does not drive GPIO pins — it only configures UART. LEDs
connected to GPIO26 and GPIO27 in `diagram.json` will light up once GPIO code
is added to the firmware. See the hardware diagram section below.

## Milestones

This project is built incrementally through five milestones. Each milestone
has a dedicated branch, an owner, and a reviewer.

| Milestone | Branch | Owner | Status |
|-----------|--------|-------|--------|
| 1 — UART Echo | feat/uart-echo | Ekojoe | ✅ Complete |
| 2 — Interrupt RX | feat/interrupt-rx | Juan | 🔄 In progress |
| 3 — Ring Buffer | feat/ring-buffer | Ekojoe | ⏳ Pending |
| 4 — Command Parser | feat/command-parser | Both | ⏳ Pending |
| 5 — Polish & Demo | feat/polish | Both | ⏳ Pending |

Full details for each milestone are in [`docs/MILESTONES.md`](docs/MILESTONES.md)
and [`docs/MILESTONES_ES.md`](docs/MILESTONES_ES.md).

## Hardware Diagram

> 🏗️ *This section will be filled once the project is closer to completion.
> It will document pin assignments, the Wokwi parts layout, and wiring
> decisions.*

For now, refer to [`diagram.json`](diagram.json) for the current Wokwi
simulation setup.

## Team

- **Ekojoe Covenant Lemom**
- **Juan Sebastian Valencia Londoño**

Repository: <https://github.com/anidroid1184/rust_demo_day>
