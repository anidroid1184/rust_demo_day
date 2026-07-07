//! UART0 controller configuration and initialization.
//! GPIO1(TX) - GPIO3(RX)

use esp_hal::Blocking;
use esp_hal::uart::{Config as UartConfig, Uart};

/// Initializes the UART0 controller at 115200 baud.
/// Uses GPIO1 (TX) and GPIO3 (RX) as default pins.
pub fn init_uart(uart0: esp_hal::peripherals::UART0<'static>) -> Uart<'static, Blocking> {
    let config = UartConfig::default().with_baudrate(115_200);

    Uart::new(uart0, config).expect("Failed to initialize UART0")
}
