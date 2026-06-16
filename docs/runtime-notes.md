# Axum Runtime Model Notes

This document explains the runtime architecture of the `ahlan-commerce` Axum server and compares it to Actix Web's worker-based model.

---

## 1. The Axum/Tokio/Hyper/Tower Architecture

Axum's stack is built on three main pillars:
- **Tokio**: The multi-threaded, work-stealing async task scheduler.
- **Hyper**: The low-level, fast HTTP server implementation.
- **Tower**: The modular middleware and service abstraction layer (`Service` trait).

### Where Do Handlers Run?
1. **Connection Acceptance**: Hyper listens for incoming TCP connections. When a connection is accepted, Tokio spawns an async task to handle the connection's lifecycle.
2. **Routing and Dispatching**: For each HTTP request, Axum matches the route and invokes the corresponding async handler function.
3. **Execution**: Handlers run as cooperative async tasks multiplexed on Tokio's worker threads (the multi-threaded scheduler). By default, Tokio spawns a pool of worker threads equal to the number of CPU cores.

---

## 2. Why Blocking Work Is Risky in Tokio

Tokio uses **cooperative scheduling**. An async task is expected to run until it hits an `.await` point, at which point it yields control back to the Tokio scheduler so other tasks can run.

If an async handler performs a **blocking operation** (e.g., `std::thread::sleep`, CPU-bound loops, synchronous file I/O, or synchronous database calls using blocking drivers):
- It **blocks the physical OS thread** (the Tokio worker thread) it is currently running on.
- Because the thread pool is small (typically matching the CPU core count), blocking even a few worker threads quickly starves the scheduler.
- Consequently, other concurrent async tasks (such as TCP handshakes, socket writes, or timers) cannot get CPU time, leading to severe latency spikes, timeout errors, or complete server hangs.

### Mitigation in Tokio
For unavoidable blocking operations, offload them to a dedicated thread pool designed for blocking tasks:
```rust
tokio::task::spawn_blocking(move || {
    // Perform blocking synchronous work here
}).await;
```

---

## 3. Axum (Tokio) vs. Actix Web Worker Model

| Comparison Dimension | Axum (Tokio Work-Stealing) | Actix Web (Worker Threads) |
| :--- | :--- | :--- |
| **Threading Model** | Single global `tokio` runtime running a work-stealing pool across physical threads. | Multiple independent worker threads, each running its own single-threaded Tokio event loop. |
| **Task Scheduling** | Tasks are scheduled globally. If a worker thread is idle, it can "steal" tasks from other busy worker threads. | Incoming TCP connections are load-balanced (round-robin) and bound to a specific worker thread. |
| **Impact of Blocking Work** | Blocking a thread reduces the global pool capacity. If all worker threads are blocked, the entire runtime halts. | Blocking a worker thread freezes that specific worker's event loop. Requests already assigned to that worker stall, but other workers continue serving new requests. |
| **State Sharing** | Share state across threads using `Arc<RwLock<T>>` or `Arc<Mutex<T>>`. | State can be instantiated per-worker thread (using `HttpServer::app_data` factory) or shared globally using `Arc`. |
