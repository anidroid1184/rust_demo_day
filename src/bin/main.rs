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
use uart_command_rust::{ring_buffer, uart_handler};

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

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    // Select UART0 (GPIO1 - TX, GPIO3 - RX) for communication
    uart_handler::init_uart(peripherals.UART0);
    let mut consumer = ring_buffer::init_ring_buffer();

    loop {
        if let Ok(read_grant) = consumer.read() {
            let len = read_grant.buf().len();

            critical_section::with(|cs| {
                if let Some(uart) = uart_handler::SHARED_UART.borrow(cs).borrow_mut().as_mut() {
                    let _ = uart.write(read_grant.buf());
                }
            });

            read_grant.release(len);
        }
    }
}

