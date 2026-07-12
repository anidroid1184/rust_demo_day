//! Main loop that reads bytes from UART0 and echoes them back.
//! Entry point of the ESP32 firmware.

#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(
    clippy::large_stack_frames,
    reason = "RAM is limited to ~320kB; a large stack allocation could overflow into \
    another context's memory. Embedded has no memory protection between stacks."
)]

use esp_hal::clock::CpuClock;
use esp_hal::main;
use rust_demo_day::uart_handler;

/// Catches unrecoverable errors and halts execution.
/// Embedded targets have nowhere to return to, so we loop forever.
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    // Clock and peripherals configuration
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Select UART0 (GPIO1 - TX, GPIO3 - RX) for communication
    let mut uart = uart_handler::init_uart(peripherals.UART0);

    // Reserve ~96kB of reclaimed SRAM for the heap allocator.
    // Required for dynamic memory (Box, Vec, etc.) in later milestones.
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    loop {
        // Single-byte buffer to hold the incoming byte
        let mut rx_byte = [0u8; 1];

        // Read one byte from UART. If successful, echo it back.
        match uart.read(&mut rx_byte) {
            Ok(_) => {
                // Send the received byte back (blocking echo)
                let _ = uart.write(&rx_byte);
            }
            Err(_) => {
                // Known limitation: RX errors (framing/overrun) are not yet
                // cleared here. esp-hal exposes check_for_rx_errors() for this,
                // but it's gated behind the `unstable` feature flag, which this
                // project doesn't currently enable. Tracked for a future pass.
            }
        }
    }
}
