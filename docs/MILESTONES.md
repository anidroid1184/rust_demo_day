# MILESTONES.md — Real-Time UART CLI for ESP32

**Team:** Ekojoe Covenant Lemom (AlphaCode) & Juan Sebastian Valencia Londoño
**Repo:** <https://github.com/anidroid1184/rust_demo_day>
**Working style:** Fully async and remote. No live sessions required. Progress is measured by branch readiness, not by time spent online together.

---

## How We Work (The Ground Rules)

- Each milestone has one **owner** who drives it to completion.
- The other person is the **reviewer** — they read the PR, leave comments, ask questions, and approve before merging.
- All communication happens over Telegram (async — no expectation of immediate replies) or GitHub PR comments.
- **Neither person should ever be blocked waiting for the other to be online.** If you're blocked, open a draft PR and describe the blocker in the description — the other person will respond when available.
- `develop` branch is always buildable. Never push broken code to `develop`.
- `main` only receives merges at the end of a completed milestone.

---

## Branch Strategy

```plain
main         ← stable, demo-ready snapshots only (tagged per milestone)
develop      ← integration branch, always buildable
feat/uart-echo       ← Milestone 1 (in progress)
feat/interrupt-rx    ← Milestone 2 (in progress)
feat/ring-buffer     ← Milestone 3 (not started)
feat/command-parser  ← Milestone 4 (not started)
feat/polish          ← Milestone 5 (not started)
```

**PR flow:** `feat/*` → `develop` (reviewed and approved by the other person) → at milestone completion, `develop` → `main` (tagged).

---

## Milestone 1 — UART Echo (Blocking)

**Branch:** `feat/uart-echo`
**Owner:** Ekojoe
**Reviewer:** Juan
**Target:** `develop`, then tag `main` as `v0.1.0`

**Goal:** Configure UART0 on the ESP32. Read one byte when it arrives, write it straight back out. Verify in Wokwi simulation using the serial monitor.

**Why this comes first:** Peripheral configuration (baud rate, TX/RX pins, FIFO settings) is a separate skill from interrupt handling. We prove the hardware wiring works on its own before adding concurrency on top of it.

**Definition of done:**

- [X] UART0 configured with correct baud rate and pins
- [X] Blocking echo working in Wokwi simulation (type a character, see it returned)
- [X] Code reviewed and approved by Juan
- [X] Merged into `develop`
- [X] `develop` merged into `main`, tagged `v0.1.0`

**Juan's role this milestone:** Review the PR. Read through the UART config code and make sure you understand how the baud rate is calculated and which pins are used — you'll be building on top of this in Milestone 2.

---

## Milestone 2 — Interrupt-Driven RX

**Branch:** `feat/interrupt-rx`
**Owner:** Juan
**Reviewer:** Ekojoe
**Target:** `develop`, then tag `main` as `v0.2.0`

**Goal:** Move UART RX from blocking to interrupt-driven. When a byte arrives, the UART peripheral fires an interrupt service routine (ISR) that captures the byte and stores it in a temporary static buffer. The main loop reads from that buffer.

**Why Juan owns this:** Juan has the physical ESP32 board. Interrupt behavior on real silicon can differ from simulation in subtle ways — having the board owner drive this milestone means we catch those differences early rather than after merging.

**Why this matters:** Blocking RX means the main loop can do nothing else while waiting for input. Interrupt-driven RX frees the main loop to stay responsive — this is the core of what makes the CLI "real-time."

**The hard part, clearly stated:** An ISR runs outside normal program flow at an unpredictable time. Rust will not allow a plain mutable reference to shared data inside an ISR — and for good reason, as it would create a data race. For this milestone we use a `critical_section::Mutex<RefCell<Option<u8>>>` as a temporary safe shared state. Milestone 3 replaces this with a proper lock-free solution.

**Definition of done:**

- [ ] UART RX handled via ISR (not blocking)
- [ ] Received bytes accessible in the main loop via the temporary static buffer
- [ ] Tested on Juan's physical ESP32 (not simulation only)
- [ ] Code reviewed and approved by Ekojoe
- [ ] Merged into `develop`
- [ ] `develop` merged into `main`, tagged `v0.2.0`

**Ekojoe's role this milestone:** Review the PR carefully. Focus on understanding the ISR registration pattern and the `critical_section` usage — this is the pattern that Milestone 3 replaces, and you need to understand it to write the replacement correctly.

---

## Milestone 3 — Lock-Free Ring Buffer

**Branch:** `feat/ring-buffer`
**Owner:** Ekojoe
**Reviewer:** Juan
**Target:** `develop`, then tag `main` as `v0.3.0`

**Goal:** Replace the temporary static buffer from Milestone 2 with a proper `bbqueue` producer/consumer split. The ISR writes bytes to the `Producer` handle. The main loop reads bytes from the `Consumer` handle.

**Why Ekojoe owns this:** This milestone is pure Rust concurrency reasoning — no hardware dependency, fully testable in Wokwi simulation. It builds directly on the ring buffer and lock-free concurrency concepts Ekojoe has been developing.

**Why `bbqueue` specifically:**

- The producer/consumer split means the ISR and the main loop each have their own non-overlapping handle to the buffer. The type system itself prevents concurrent access — no mutex, no disabling interrupts, no runtime cost.
- `no_std` rules out heap-based structures like `VecDeque` unless we add an allocator, which adds complexity we don't need.
- Manual ring buffer implementations frequently have subtle wraparound bugs. `bbqueue` has already solved this correctly.

**Definition of done:**

- [ ] `bbqueue` added to `Cargo.toml`
- [ ] `Producer` handle passed to ISR, `Consumer` handle used in main loop
- [ ] Temporary static buffer from Milestone 2 fully removed
- [ ] Wokwi simulation: bytes typed in serial monitor arrive correctly in main loop
- [ ] Code reviewed and approved by Juan
- [ ] Merged into `develop`
- [ ] `develop` merged into `main`, tagged `v0.3.0`

**Juan's role this milestone:** Review the PR. Focus on understanding why the producer/consumer split removes the need for the `critical_section` mutex from Milestone 2 — this is a key concept for the defense.

---

## Milestone 4 — Command Parser

**Branch:** `feat/command-parser`
**Owner:** Both (split as described below)
**Target:** `develop`, then tag `main` as `v0.4.0`

**Goal:** The main loop drains bytes from the `Consumer`, accumulates them into a line buffer until it sees a newline (`\n`), splits on whitespace, matches the first word against known commands, and writes a response back over TX.

**Split within this milestone:**

- **Ekojoe:** Line accumulator + command dispatcher scaffolding. Write the function signatures and the dispatch table that maps command names to handler functions. Commit this to the branch first so Juan has a clear interface to build against.
- **Juan:** Implement the actual command handlers. At minimum: `ping` → responds `pong`, `help` → lists available commands, `status` → responds with a short chip status message.

**Why fixed-size buffers:** `no_std` has no heap-backed `String` by default. We use `heapless::String<64>` or a plain `[u8; 64]` array as the line buffer — fixed size, stack-allocated, no allocator needed.

**Definition of done:**

- [ ] Line accumulator working (bytes accumulate until `\n`)
- [ ] At least 3 commands implemented and responding correctly (`ping`, `help`, `status`)
- [ ] Unknown command returns a clear error response
- [ ] Tested in Wokwi simulation
- [ ] Both sides reviewed and approved before merging
- [ ] Merged into `develop`
- [ ] `develop` merged into `main`, tagged `v0.4.0`

---

## Milestone 5 — Polish + Demo Prep

**Branch:** `feat/polish`
**Owner:** Both
**Target:** `develop` → `main`, tagged `v1.0.0-demo`

**Goal:** Make the project demo-ready and defensible.

**Tasks:**

- [ ] Backspace handling in the line accumulator
- [ ] Character echo as the user types (terminal feels alive)
- [ ] Clean, informative responses to all edge cases
- [ ] `README.md` updated with setup instructions, demo instructions, and project description
- [ ] `ARCHITECTURE.md` filled in with the final data flow diagram and design decision rationale
- [ ] Wokwi `diagram.json` cleaned up and presentable
- [ ] Demo video recorded (1-2 minutes, showing the CLI responding to commands in simulation)
- [ ] Final merge to `main`, tagged `v1.0.0-demo`

---

## Interface Contract (Agreed Before Milestone 3 Starts)

Before Juan starts Milestone 2 and Ekojoe starts Milestone 3, we agree on this interface in writing:

The ISR will call:

```rust
// producer is a bbqueue Producer handle accessible to the ISR
if let Ok(mut grant) = producer.grant_exact(1) {
    grant[0] = received_byte;
    grant.commit(1);
}
```

The main loop will call:

```rust
if let Ok(grant) = consumer.read() {
    for byte in grant.buf() {
        // process byte
    }
    let len = grant.buf().len();
    grant.release(len);
}
```

Both of us commit to this interface before either branch diverges significantly — it means Juan can write the ISR side and Ekojoe can write the consumer side without needing to sync up live.

---

## Summary Table

| Milestone | Branch | Owner | Reviewer | Main Tag |
| --------- | ------ | ----- | -------- | -------- |
| 1 — UART Echo | feat/uart-echo | Ekojoe | Juan | v0.1.0 |
| 2 — Interrupt RX | feat/interrupt-rx | Juan | Ekojoe | v0.2.0 |
| 3 — Ring Buffer | feat/ring-buffer | Ekojoe | Juan | v0.3.0 |
| 4 — Parser | feat/command-parser | Both | Both | v0.4.0 |
| 5 — Polish | feat/polish | Both | Both | v1.0.0-demo |
