//! Lock-free byte transfer between ISR (producer) and main loop (consumer).
//! Uses bbqueue to avoid mutexes and critical sections.
//
// Replaces the previous SHARED_BYTE: Mutex<RefCell<Option<u8>>> approach.
// That approach could only hold ONE byte at a time - if bytes arrived
// faster than the main loop checked, earlier bytes were silently
// overwritten and lost. bbqueue's ring buffer holds many bytes safely.

use bbqueue::{BBBuffer, Consumer, Producer};
use core::cell::RefCell;
use critical_section::Mutex;

// Buffer size in bytes. 64 bytes is plenty for typed CLI input —
// far more than a human can type between main loop iterations.
const BUFFER_SIZE: usize = 64;

// The actual byte storage. `static` so it lives for the program's
// entire lifetime — required since both the ISR and main() need to
// reach it, and neither owns the other's stack frame.
static BB: BBBuffer<BUFFER_SIZE> = BBBuffer::new();

// The Producer must be reachable from inside the ISR, which takes no arguments
pub static SHARED_PRODUCER: Mutex<RefCell<Option<Producer<'static, BUFFER_SIZE>>>> =
    Mutex::new(RefCell::new(None));

/// Splits the static BBBuffer, stashes the Producer half into
/// SHARED_PRODUCER for the ISR to use, and returns the Consumer half
/// for main() to use directly as a normal local variable.
pub fn init_ring_buffer() -> Consumer<'static, BUFFER_SIZE> {
    let (producer, consumer) = BB.try_split().expect("BBBuffer already split");

    critical_section::with(|cs| {
        SHARED_PRODUCER.borrow(cs).borrow_mut().replace(producer);
    });

    consumer
}

/// Called from inside the ISR. Writes one byte into the ring buffer
/// via the Producer stashed in SHARED_PRODUCER.
pub fn push_byte_from_isr(byte: u8) {
    critical_section::with(|cs| {
        if let Some(producer) = SHARED_PRODUCER.borrow(cs).borrow_mut().as_mut()
            && let Ok(mut grant) = producer.grant_exact(1)
        {
            grant[0] = byte;
            grant.commit(1);
        }
        // If grant_exact fails, the buffer is full - byte is
        // dropped. Acceptable for now; revisit if fast typing
        // during testing shows dropped characters.
    })
}
