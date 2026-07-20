//! UART0 controller configuration and initialization.
//! GPIO1(TX) - GPIO3(RX)

use core::cell::RefCell;
use critical_section::Mutex;
use esp_hal::{
    Blocking,
    peripherals::UART0,
    uart::{Config, RxConfig, Uart, UartRx, UartTx},
};

// Option: Somebyte or none
// RefCell: Allow us to modify the value of a static var
// Mutex: Only one context per time (or ISR or Loop)
pub static SHARED_BYTE: Mutex<RefCell<Option<u8>>> = Mutex::new(RefCell::new(None));
pub static SHARED_RX: Mutex<RefCell<Option<UartRx<'static, Blocking>>>> =
    Mutex::new(RefCell::new(None));

// Initializes the UART0 controller at 115200 baud.
// return a uart object with liste_rx() charged
// Uses GPIO1 (TX) and GPIO3 (RX) as default pins.
pub fn init_uart(uart0: UART0<'static>) -> UartTx<'static, Blocking> {
    let config = Config::default()
        .with_baudrate(115_200)
        .with_rx(RxConfig::default().with_fifo_full_threshold(1));

    let mut uart = Uart::new(uart0, config).expect("Failed to initialize UART0");

    // set who event read
    uart.set_interrupt_handler(interruption);
    // read the especific event
    uart.listen(esp_hal::uart::UartInterrupt::RxFifoFull);

    let (uart_rx, uart_tx) = uart.split();
    // save the uart on global mutex
    critical_section::with(|cs| {
        SHARED_RX.borrow(cs).borrow_mut().replace(uart_rx);
    });

    uart_tx
}

// execute on ISR
#[esp_hal::handler]
pub fn interruption() {
    critical_section::with(|cs| {
        if let Some(uart) = SHARED_RX.borrow(cs).borrow_mut().as_mut() {
            let mut buffer = [0u8; 1];

            if let Ok(bytes_read) = uart.read_buffered(&mut buffer)
                && bytes_read > 0
            {
                // borrow(cs): access to RefCell of secure form with cs token
                // borrow_mut(): Give a mutable reference of Option<u8>
                // replace(readed_byte): save the readed by of secure way
                SHARED_BYTE.borrow(cs).borrow_mut().replace(buffer[0]);
            }

            // uart.clear_interrupts(esp_hal::uart::UartInterrupt::RxFifoFull.into());
        }
    });
}
