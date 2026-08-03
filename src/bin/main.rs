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
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::time::{Duration, Instant};
use esp_hal::main;
use uart_command_rust::uart_handler;

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
    let mut blue_led = Output::new(peripherals.GPIO26, Level::High, OutputConfig::default());

    // Reserve ~96kB of reclaimed SRAM for the heap allocator.
    // Required for dynamic memory (Box, Vec, etc.) in later milestones.
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    // print only one time the message
    let _ = uart.write(b"\n\rstarting...");
    loop {
        // read share buffer to take byte sended
        let read_byte = critical_section::with(|cs| {
            // return the taked byte
            // IMPORTANT: don't add a ";" at the end
            uart_command_rust::uart_handler::SHARED_BYTE
                .borrow(cs)
                .borrow_mut()
                .take()
        });

        // interruption
        if let Some(byte) = read_byte {
            let _ = uart.write(b"\n\rwaiting byte...");
            // temporal buffer
            let rx_byte = [byte];
            let _ = uart.write(b"\nreadbyte: ");
            let _ = uart.write(&rx_byte);
        }

        blue_led.toggle();
        blocking_delay(Duration::from_millis(500));
    }
}

fn blocking_delay(duration: Duration) {
    let delay_start = Instant::now();
    while delay_start.elapsed() < duration {}
}