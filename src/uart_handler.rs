//! UART0 controller configuration and initialization.
//! GPIO1(TX) - GPIO3(RX)

use core::cell::RefCell;
use critical_section::Mutex;
use esp_hal::{
    Blocking,
    peripherals::UART0,
    uart::{Config, RxConfig, Uart},
};

use crate::ring_buffer;

// Option: Somebyte or none
// RefCell: Allow us to modify the value of a static var
// Mutex: Only one context per time (or ISR or Loop)
pub static SHARED_UART: Mutex<RefCell<Option<Uart<'static, Blocking>>>> =
    Mutex::new(RefCell::new(None));

// Initializes the UART0 controller at 115200 baud.
// return a uart object with liste_rx() charged
// Uses GPIO1 (TX) and GPIO3 (RX) as default pins.
pub fn init_uart(uart0: UART0<'static>) {
    let config = Config::default()
        .with_baudrate(115_200)
        .with_rx(RxConfig::default().with_fifo_full_threshold(1));

    let mut uart = Uart::new(uart0, config).expect("Failed to initialize UART0");

    uart.set_interrupt_handler(interruption);
    uart.listen(esp_hal::uart::UartInterrupt::RxFifoFull);

    critical_section::with(|cs| {
        SHARED_UART.borrow(cs).borrow_mut().replace(uart);
    });
}

// execute on ISR
#[esp_hal::handler]
pub fn interruption() {
    critical_section::with(|cs| {
        if let Some(uart) = SHARED_UART.borrow(cs).borrow_mut().as_mut() {
            let mut buffer = [0u8; 1];

            if let Ok(bytes_read) = uart.read_buffered(&mut buffer)
                && bytes_read > 0
            {
                // push into the ring buffer instead of a single slot
                ring_buffer::push_byte_from_isr(buffer[0]);
            }

            uart.clear_interrupts(esp_hal::uart::UartInterrupt::RxFifoFull.into());
        }
    });
}

