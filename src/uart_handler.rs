// UART0 peripheral configuration
// Responsible for: baud rate setup, TX/RX pin assignment, and later ISR registration
use esp_hal::Blocking;
use esp_hal::uart::{Config as UartConfig, Uart};

pub fn init_uart(uart0: esp_hal::peripherals::UART0<'static>) -> Uart<'static, Blocking> {
    let config = UartConfig::default().with_baudrate(115_200);

    Uart::new(uart0, config).expect("Failed to initialize UART0")
}
