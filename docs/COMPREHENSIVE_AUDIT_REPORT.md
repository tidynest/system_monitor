# Comprehensive Codebase Audit Report

**Project:** system_monitor
**Branch:** production
**Date:** 2026-02-17
**Scope:** Full codebase audit across 5 domains: Core Logic, Data Collection, Display/UI, Build System, and Documentation
**Total Findings:** 72 (deduplicated to 58 unique issues)

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Critical Bugs (Immediate Action)](#critical-bugs)
3. [Architecture Issues](#architecture-issues)
4. [Data Collection & Processing Issues](#data-collection--processing-issues)
5. [Display, UI & Frontend Issues](#display-ui--frontend-issues)
6. [Security Concerns](#security-concerns)
7. [Build System & Testing Issues](#build-system--testing-issues)
8. [Documentation Inconsistencies](#documentation-inconsistencies)
9. [Prioritized Fix Plan](#prioritized-fix-plan)
10. [Full Findings Index](#full-findings-index)

---

## Executive Summary

The system_monitor codebase is a ~2,100-line Rust application providing real-time system monitoring via a web dashboard. The architecture follows a collector->model->service->route pipeline with SSE streaming and HTMX polling.

**The audit found 5 critical bugs, 15 significant issues, 17 warnings, and 21 suggestions.** The most severe problems centre around the concurrency architecture: a global `std::sync::Mutex` blocks the async runtime with thread sleeps, can permanently crash the server on mutex poisoning, and creates resource contention under concurrent load. Additionally, two configuration options (`MONITOR_UPDATE_INTERVAL`, `MONITOR_MAX_PROCESSES`) are parsed, logged, but never actually wired into the application logic.

### Severity Breakdown

| Severity | Count | Description |
|----------|-------|-------------|
| CRITICAL | 5 | Crashes, data corruption, resource leaks |
| ISSUE | 15 | Incorrect behaviour, broken features, security |
| WARNING | 17 | Suboptimal behaviour, potential problems |
| SUGGESTION | 21 | Improvements, best practices, missing features |

---

## Critical Bugs

### CRIT-01: Global Mutex Blocks Async Runtime with Thread Sleeps

**Files:** `src/collectors/system.rs:48,79`
**Severity:** CRITICAL

`collect_system_metrics()` calls `SYSTEM_STATE.lock().unwrap()` (a `std::sync::Mutex`) and holds it across:
- Multiple `sysinfo` refresh calls reading from `/proc` (I/O-bound)
- An explicit `std::thread::sleep(Duration::from_millis(100))` on line 79
- All collector function iterations

This function runs on actix-web's tokio async runtime. A `std::sync::Mutex` held across a blocking sleep on an async worker thread blocks that entire tokio worker thread. With actix-web's default thread pool (equal to CPU cores), 4-8 simultaneous SSE clients on a 4-core machine can starve all worker threads, making the server completely unresponsive.

**Fix approach:** Redesign to use a background `tokio::task` that collects metrics on a timer using `spawn_blocking`, publishing results via a `tokio::sync::watch` channel. SSE handlers subscribe to the channel with zero blocking.

---

### CRIT-02: Mutex Poison Panic Crashes Entire Server

**File:** `src/collectors/system.rs:48`
**Severity:** CRITICAL

```rust
let mut state = SYSTEM_STATE.lock().unwrap();
```

If any thread panics while holding `SYSTEM_STATE`, the mutex becomes poisoned. Every subsequent `.lock().unwrap()` panics, crashing all actix-web worker threads. Since this function is called on every SSE tick and HTTP request, a single transient panic permanently kills the server until restart.

**Fix approach:** Replace with `.lock().unwrap_or_else(|e| e.into_inner())` or use `parking_lot::Mutex` (which doesn't poison).

---

### CRIT-03: Memory Division by Zero Produces NaN, Breaks JSON Serialization

**File:** `src/collectors/memory.rs:22`
**Severity:** CRITICAL

```rust
usage_percent: (used_memory / total_memory) * 100.0,
```

If `sys.total_memory()` returns 0 (containerized environments, certain VMs, unreadable `/proc/meminfo`), this produces `NaN`. `serde_json` rejects `NaN` by default, causing `format_sse_update()` to fail on every tick. The stream silently degrades to heartbeats forever with no user-visible error.

Note: The disk collector at `src/collectors/disk.rs:21` already has the correct guard pattern.

**Fix approach:** Add zero-check: `if total_memory > 0.0 { (used_memory / total_memory) * 100.0 } else { 0.0 }`

---

### CRIT-04: Network Metrics Show Near-Zero Values (New Instance Each Call)

**Files:** `src/collectors/network.rs:13`, `src/collectors/disk.rs:13`
**Severity:** CRITICAL (network), LOW (disk)

```rust
let networks = Networks::new_with_refreshed_list();  // network.rs
let disks = Disks::new_with_refreshed_list();        // disk.rs
```

Both collectors create brand-new `sysinfo` instances on every call. For networks, `total_received()`/`total_transmitted()` return cumulative totals *since the instance was created*. Since a new instance is created each second, values reset every call -- users see near-zero or meaningless lifetime-boot-total values rather than actual throughput.

**Fix approach:** Move `Networks` and `Disks` into the `SystemState` singleton. Refresh in-place. Compute delta-based rates (bytes/sec) between refreshes.

---

### CRIT-05: SSE Reconnection Loop Stops After First Failed Retry

**File:** `static/html/dashboard.html:491-501`
**Severity:** CRITICAL

The reconnection logic uses a `reconnectTimer` guard to prevent scheduling multiple timeouts. However, `reconnectTimer` is never set to `null` after the timeout fires. When the first retry fails, `onerror` fires again, sees the stale (truthy) timeout ID, and skips scheduling a new retry. The dashboard permanently shows "Disconnected" with no further attempts -- the user must manually refresh.

**Fix approach:** Set `reconnectTimer = null` at the beginning of the timeout callback, before calling `connect()`.

---

## Architecture Issues

### ARCH-01: Config Values Parsed But Never Used

**Files:** `src/config/mod.rs:60-73`, `src/main.rs:38-39`, `src/routes/metrics.rs:35`, `src/collectors/process.rs:48,51`
**Severity:** ISSUE

`MONITOR_UPDATE_INTERVAL` and `MONITOR_MAX_PROCESSES` are read from environment, stored in `Config`, and logged at startup, but:
- SSE interval hardcoded to `Duration::from_secs(1)` in `metrics.rs:35`
- Process limit hardcoded to `5` in `process.rs:48,51` and `metrics.rs:140`
- Disk display hardcoded to `3` in `metrics.rs:164`
- `Config` is never passed as `web::Data` to route handlers

Users who set these env vars see no effect.

**Fix approach:** Register `Config` as `web::Data<Config>` in `main.rs` and consume values in handlers.

### ARCH-02: Redundant Full Metric Collection Per Request

**Files:** `src/routes/metrics.rs`, `src/services/metrics_service.rs`
**Severity:** ISSUE

Each endpoint (`/metrics/stream`, `/metrics/processes`, `/metrics/disks`, `/metrics/network`) calls `metrics_service.collect()`, which performs a **full** system collection (CPU, memory, disk, network, processes) including mutex lock and potential sleep. The dashboard triggers ~2.7 full collections per second per client. One cached snapshot would suffice.

**Fix approach:** Background collection task publishes to a `watch` channel; all endpoints read the latest snapshot without re-collecting.

### ARCH-03: Triple Process Refresh Per Collection Cycle

**Files:** `src/collectors/system.rs:69-86`, `src/collectors/process.rs:24`
**Severity:** ISSUE

`collect_system_metrics()` refreshes processes twice (lines 69-73 and 76-86 with conditional sleep), then calls `collect_process_metrics()` which refreshes a third time (line 24). Each refresh iterates every `/proc/[pid]/` entry.

**Fix approach:** Remove the refresh from `collect_process_metrics()`. Make it a pure data-extraction function.

### ARCH-04: Lazy Static Init Blocks First Request

**File:** `src/collectors/system.rs:23-41`
**Severity:** WARNING

The `Lazy::new` initializer sleeps 200ms + does `System::new_all()` (potentially hundreds of ms more). First client request sees 400-700ms latency.

**Fix approach:** Initialize eagerly in `main()` before server bind.

### ARCH-05: Static Files Served From Relative Path

**File:** `src/main.rs:57`
**Severity:** WARNING

```rust
Files::new("/static", "./static")
```

If launched from a different working directory (systemd, deployment scripts), static assets 404. The dashboard HTML loads via `include_str!` (compiled in), creating an inconsistency.

**Fix approach:** Embed static assets at compile time, or derive path from executable location.

### ARCH-06: Log Warnings Emitted Before Logger Initialization

**File:** `src/config/mod.rs:56,63,70,81` vs `src/main.rs:21-26`
**Severity:** WARNING

`Config::from_env()` calls `log::warn!()` for invalid env values, but is called on `main.rs:21` **before** `env_logger::init()` on `main.rs:24-26`. Configuration warnings are silently discarded.

**Fix approach:** Collect warnings as `Vec<String>` during config parse, log after logger init.

---

## Data Collection & Processing Issues

### DATA-01: Network Shows Cumulative Totals, Not Throughput Rates

**File:** `src/collectors/network.rs:20-21`
**Severity:** ISSUE

`total_received()`/`total_transmitted()` return boot-lifetime cumulative byte counts. Dashboard shows "150,000 MB" total rather than "50 MB/s" throughput. Combined with CRIT-04 (new instance each call), the values are doubly incorrect.

**Fix approach:** Persist `Networks` in global state, use `received()`/`transmitted()` (per-interval deltas) to compute rates.

### DATA-02: CPU-Intensive Low-Memory Processes Hidden

**File:** `src/collectors/process.rs:33-37`
**Severity:** WARNING

The 10MB memory filter applies to ALL processes before building both `top_cpu` and `top_memory` lists. A tight computational loop in a small binary (<10MB) will never appear in `top_cpu`.

**Fix approach:** Apply the 10MB filter only to `top_memory`. Build `top_cpu` from all processes.

### DATA-03: Aggressive Memory Deduplication Suppresses Legitimate Consumers

**File:** `src/collectors/process.rs:75-98`
**Severity:** WARNING

`get_top_unique_by_memory` rounds to nearest MB and skips duplicates. If 20 browser tabs each use 100.x MB, only 1 appears. The list may show 1 process instead of the requested 5.

**Fix approach:** Remove uniqueness filter or use finer granularity (0.1 MB).

### DATA-04: `core_count` Reports Logical CPUs, Not Physical Cores

**File:** `src/collectors/cpu.rs:26`
**Severity:** WARNING

`sys.cpus().len()` returns logical processors (including hyperthreads). A 6-core/12-thread CPU shows as 12 "cores".

**Fix approach:** Rename to `logical_cpu_count` or add separate `physical_core_count` field.

### DATA-05: CPU Frequency From First Core Only

**File:** `src/collectors/cpu.rs:17-21`
**Severity:** WARNING

On hybrid architectures (Intel P-cores/E-cores), first core may not represent overall frequency.

**Fix approach:** Report min/max/average across all cores.

### DATA-06: Disk Usage Includes Reserved Blocks

**File:** `src/collectors/disk.rs:18-25`
**Severity:** WARNING

`used = total - available` includes ext4 reserved-for-root space (~5%). Dashboard may show higher usage than `df`.

**Fix approach:** Document the behaviour or compute `used / (used + available)` for df-equivalent percentage.

### DATA-07: Timestamp Lacks Timezone Information

**File:** `src/collectors/system.rs:94`
**Severity:** SUGGESTION

`chrono::Local::now().format("%Y-%m-%d %H:%M:%S")` produces an ambiguous string with no timezone.

**Fix approach:** Use `to_rfc3339()` or include a `timestamp_epoch` numeric field.

### DATA-08: No Percentage Clamping

**Files:** All collectors computing percentages
**Severity:** SUGGESTION

No collector clamps output to `[0.0, 100.0]`. Edge cases could break frontend progress bars.

**Fix approach:** Add `.clamp(0.0, 100.0)` to all computed percentages.

### DATA-09: Mixed f32/f64 Precision Across Models

**Files:** `src/models/cpu.rs` (f32), `src/models/memory.rs` (f64), `src/models/process.rs` (mixed)
**Severity:** SUGGESTION

CPU uses f32 (from sysinfo), memory uses f64. JSON consumers see inconsistent precision.

**Fix approach:** Standardise to f64 at the collection boundary.

### DATA-10: Missing swap_usage_percent Field

**File:** `src/models/memory.rs`
**Severity:** SUGGESTION

Model has `swap_total_gb` and `swap_used_gb` but no percentage field.

### DATA-11: Inverted Sleep Condition

**File:** `src/collectors/system.rs:76-86`
**Severity:** ISSUE

The condition `now.duration_since(state.last_process_refresh) < Duration::from_millis(100)` triggers a 100ms blocking sleep when the previous refresh was very recent. Under concurrent HTTP requests, this introduces unnecessary 100ms blocking delays while holding the global mutex.

**Fix approach:** Track whether CPU deltas are established; skip sleep once calibrated.

---

## Display, UI & Frontend Issues

### UI-01: Debug Utility `sseDebug.raw()` Permanently Breaks Dashboard

**File:** `static/html/dashboard.html:550-558`
**Severity:** BUG

Calling `sseDebug.raw()` permanently replaces `evtSource.onmessage` with a console-log-only handler. Dashboard stops updating with no visible indication. Status bar still shows "Connected".

**Fix approach:** Remove debug utilities from production, or save/restore the original handler.

### UI-02: Native EventSource Reconnect Fights Custom Reconnect

**File:** `static/html/dashboard.html:464,491-501`
**Severity:** WARNING

Browser's built-in `EventSource` auto-reconnect runs simultaneously with the custom `setTimeout`-based reconnect. The native reconnect may succeed, then the manual timeout fires 3 seconds later, destroys the working connection, and creates another.

**Fix approach:** Choose one strategy. Either rely on native reconnect (remove manual `connect()` call) or close EventSource in `onerror` before scheduling manual reconnect.

### UI-03: No Exponential Backoff on Reconnection

**File:** `static/html/dashboard.html:497`
**Severity:** WARNING

Fixed 3-second retry. Many simultaneous clients create thundering herd on server restart.

**Fix approach:** Exponential backoff with jitter (1s -> 2s -> 4s, cap 30s, +/- 20% jitter).

### UI-04: HTMX SSE Extension Loaded But Unused

**File:** `static/html/dashboard.html:14`
**Severity:** WARNING

`sse.js` extension loaded from CDN but never referenced. No `hx-ext="sse"` attribute anywhere. Real SSE handled by custom vanilla JS.

**Fix approach:** Remove the `sse.js` script tag.

### UI-05: CDN SSE Extension Lacks Subresource Integrity

**File:** `static/html/dashboard.html:14`
**Severity:** WARNING

Main HTMX has SRI (`integrity` attribute), but the SSE extension does not. Supply chain risk.

**Fix approach:** Add `integrity` and `crossorigin="anonymous"`, or bundle scripts locally.

### UI-06: No Stale Data Indication After Disconnect

**File:** `static/html/dashboard.html`
**Severity:** ISSUE

When SSE disconnects, metrics freeze at last values. "Disconnected" text in status bar is easy to miss. No visual indication on metric cards that data is stale.

**Fix approach:** Add visual overlay, staleness badge, or reset values to "--" after timeout.

### UI-07: HTMX Polling Has No Error Handling

**File:** `static/html/dashboard.html:268-273`
**Severity:** WARNING

No `hx-on::error` handlers. When server is down, HTMX-polled sections (processes, disks, network) show stale data silently.

**Fix approach:** Add `htmx:responseError` event listener to show "Connection lost" message.

### UI-08: Progress Bar Initial Width Missing CSS Unit

**File:** `static/html/dashboard.html:238,254`
**Severity:** ISSUE

`style="width: 0"` -- missing unit. Should be `width: 0%`.

### UI-09: Inline Styles in Server-Rendered HTML Fragments

**File:** `src/routes/metrics.rs:147-213`
**Severity:** WARNING

Extensive inline styles in Rust HTML fragments (`style="color: #667eea; font-weight: bold;"`) while dashboard.html has a proper CSS section. Creates maintenance burden when changing colour scheme.

**Fix approach:** Define CSS classes in dashboard.html, reference them in Rust fragments.

### UI-10: Excessive Console Logging in Production

**File:** `static/html/dashboard.html` (11+ calls)
**Severity:** SUGGESTION

`console.log('[SSE] DOM updated successfully')` fires every second. On long-running sessions, console accumulates thousands of entries.

**Fix approach:** Remove production `console.log` calls. Keep only `console.error` for real errors.

### UI-11: html_escape Allocates Per Character

**File:** `src/routes/metrics.rs:247-257`
**Severity:** SUGGESTION

Creates a new `String` for every character. Called 13+ times per polling cycle.

**Fix approach:** Use `String::with_capacity` and `push`/`push_str`, or use `askama_escape` crate.

### UI-12: Collected Data Never Displayed

**Files:** `src/models/cpu.rs:18-19`, `src/routes/metrics.rs:221-226`
**Severity:** WARNING

`cpu.brand`, `cpu.per_core_usage`, `disk.name`, `disk.file_system`, `network.interfaces`, and `process.total_count` are collected but never displayed or transmitted.

**Fix approach:** Either display these fields or remove from collection to save resources.

### UI-13: Process Name Overflow on Narrow Screens

**File:** `src/routes/metrics.rs:147-154`
**Severity:** SUGGESTION

No `max-width`, `overflow: hidden`, or `text-overflow: ellipsis` on process name spans.

---

## Accessibility Issues

### A11Y-01: Progress Bars Lack ARIA Attributes

**File:** `static/html/dashboard.html:237-239`
**Severity:** ISSUE

Progress bars are plain `<div>` elements. Screen readers cannot perceive them as progress indicators.

**Fix approach:** Add `role="progressbar"`, `aria-valuenow`, `aria-valuemin`, `aria-valuemax`, `aria-label`.

### A11Y-02: Emoji Icons Confuse Screen Readers

**File:** `static/html/dashboard.html:234,249,266,280`
**Severity:** ISSUE

Screen readers announce "high voltage CPU Usage" or "floppy disk Memory".

**Fix approach:** Add `aria-hidden="true"` to icon spans.

### A11Y-03: No Semantic HTML Landmarks

**File:** `static/html/dashboard.html`
**Severity:** WARNING

Uses `<div class="header">` instead of `<header>`. No `<main>`, `<nav>`, or skip-navigation.

---

## Security Concerns

### SEC-01: Wildcard CORS on SSE Endpoint

**File:** `src/routes/metrics.rs:67`
**Severity:** ISSUE

```rust
.insert_header(("Access-Control-Allow-Origin", "*"))
```

Any website can open an EventSource to this endpoint and read system metrics (CPU, memory, process names, disk mounts, hostname). Process names reveal running software. Dashboard is same-origin, so CORS header is unnecessary.

**Fix approach:** Remove the wildcard CORS header entirely.

### SEC-02: Hand-Rolled HTML Escape Function

**File:** `src/routes/metrics.rs:247-257`
**Severity:** WARNING

Custom escape covers 5 standard characters but not null bytes, control characters, or Unicode edge cases. Process names come from the OS and could contain unexpected sequences. Also has zero test coverage.

**Fix approach:** Use a well-tested library (`askama_escape`, `html_escape` crate). Add tests.

### SEC-03: `env::set_var`/`env::remove_var` in Tests is Undefined Behaviour

**File:** `src/config/mod.rs:127-130`
**Severity:** BUG

As of Rust 1.83, these functions are deprecated and marked `unsafe` because they cause undefined behaviour when multiple threads access environment variables concurrently. `cargo test` runs tests in parallel by default.

**Fix approach:** Use `temp_env` crate, or `serial_test`, or refactor `Config::from_env()` to accept an env-reader parameter.

---

## Build System & Testing Issues

### BUILD-01: No CI/CD Pipeline

**Location:** Project root (no `.github/workflows/`)
**Severity:** ISSUE

No automated compilation, testing, linting, or security scanning. `.gitignore` references `workflows/*.yml.disabled`, suggesting CI was considered but never implemented.

**Fix approach:** Create `.github/workflows/ci.yml` with `cargo check`, `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check`, `cargo audit`.

### BUILD-02: Zero Test Coverage on HTTP Layer

**Location:** `src/routes/` (all handlers), `src/services/`
**Severity:** ISSUE

22 test functions exist, but ALL are unit tests in `#[cfg(test)]` modules. Zero integration tests. The entire HTTP layer (5 route handlers), SSE stream formatting, HTML rendering functions, and `html_escape` have no tests.

**Most critical untested code:**
- `html_escape()` -- security-critical XSS prevention, zero tests
- `format_sse_update()` -- SSE protocol formatting, zero tests
- All route handlers -- zero response verification

**Fix approach:** Create `tests/` directory with `actix-test` integration tests. Prioritise `html_escape` and `format_sse_update`.

### BUILD-03: `once_cell` Dependency Is Unnecessary

**Files:** `Cargo.toml:28`, `src/collectors/system.rs:7`
**Severity:** ISSUE

Since Rust 1.80, `std::sync::LazyLock` provides identical functionality in the standard library.

**Fix approach:** Replace `once_cell::sync::Lazy` with `std::sync::LazyLock`, remove from Cargo.toml.

### BUILD-04: `tokio` Features Too Broad

**File:** `Cargo.toml:9`
**Severity:** WARNING

`features = ["full"]` enables `process`, `signal`, `fs`, `net`, `parking_lot`, etc. -- all unused. Increases binary size, compile time, and attack surface.

**Fix approach:** `features = ["rt-multi-thread", "macros", "time"]`

### BUILD-05: No `[profile.release]` Configuration

**File:** `Cargo.toml`
**Severity:** ISSUE

Default release settings. No LTO, no strip, 16 codegen units.

**Fix approach:** Add `[profile.release]` with `opt-level = 3`, `lto = "thin"`, `codegen-units = 1`, `strip = true`.

### BUILD-06: Empty `[dev-dependencies]` With Dangling Comment

**File:** `Cargo.toml:31-32`
**Severity:** ISSUE

```toml
[dev-dependencies]
# For testing async code
```

Suggests dev-dependencies were planned but never added. No async test infrastructure.

### BUILD-07: Test Function Misnamed in Network Module

**File:** `src/collectors/network.rs:44`
**Severity:** WARNING

`test_cpu_metrics_collection` tests `collect_network_metrics()`. Copy-paste error.

### BUILD-08: No `rustfmt.toml` or `clippy.toml`

**Location:** Project root
**Severity:** SUGGESTION

No enforced code style. Different editors may format differently.

### BUILD-09: No `rust-toolchain.toml`

**Location:** Project root
**Severity:** SUGGESTION

No pinned Rust version for reproducible builds.

### BUILD-10: Redundant `#[allow(dead_code)]` on `format_bytes`

**File:** `src/utils/mod.rs:13`
**Severity:** WARNING

Function is `pub` (dead_code lint doesn't fire for public items). The attribute is redundant and masks the fact that the function is never called.

### BUILD-11: No Benchmarks

**Location:** Project root
**Severity:** SUGGESTION

No `benches/` directory. No way to track performance regressions in metric collection.

---

## Documentation Inconsistencies

### DOC-01: Version Mismatch

**Files:** `Cargo.toml:4` says `0.1.0`, `README.md:240` says `v1.0.2`
**Severity:** BUG

Canonical version authority conflict.

### DOC-02: "Zero JavaScript Dependencies" Claim Inaccurate

**Files:** `README.md:18`, `static/html/dashboard.html:10-14`
**Severity:** ISSUE

Dashboard loads HTMX + SSE extension from CDN, plus ~260 lines of custom JavaScript.

**Fix approach:** Reword to "No build-time JavaScript toolchain required".

### DOC-03: README Omits `trace` as Valid Log Level

**Files:** `README.md:51`, `src/config/mod.rs:77`
**Severity:** WARNING

Config validates `trace` as valid but README only lists `error/warn/info/debug`.

### DOC-04: Inconsistent/Placeholder Repository URLs

**File:** `README.md:29,205`
**Severity:** ISSUE

Line 29: `github.com/yourusername/system-monitor.git` (placeholder)
Line 205: `github.com/tidynest/system_monitor/issues` (appears real, different name format)

### DOC-05: SSE Interval Hardcoded Contradicts Config Documentation

**Files:** `README.md:15,49`, `src/routes/metrics.rs:35`
**Severity:** WARNING

README documents `MONITOR_UPDATE_INTERVAL` config option, but SSE interval is hardcoded to 1 second.

### DOC-06: "All Mounted Filesystems" Claim vs Hardcoded `.take(3)`

**Files:** `README.md:79`, `src/routes/metrics.rs:164`
**Severity:** WARNING

README says "all mounted filesystems" but only 3 are ever shown.

### DOC-07: Incomplete Comment Cut Off Mid-Sentence

**File:** `src/collectors/system.rs:112`
**Severity:** WARNING

`// Allow system to ` -- comment truncated.

### DOC-08: "Production Ready" Claim vs `unwrap()` Usage

**Files:** `README.md:17`, `src/collectors/system.rs:48`
**Severity:** WARNING

README claims "comprehensive error handling" while mutex access uses `unwrap()`.

### DOC-09: Missing Cargo.toml Metadata

**File:** `Cargo.toml`
**Severity:** SUGGESTION

Missing `description`, `license`, `authors`, `repository`, `keywords`, `rust-version`.

### DOC-10: No CHANGELOG

**Location:** Project root
**Severity:** SUGGESTION

README references v1.0.2 implying multiple releases, but no changelog exists.

### DOC-11: API Docs Omit Response Formats and Query Parameters

**File:** `README.md:136-144`
**Severity:** SUGGESTION

Endpoints return HTML fragments (not JSON). `/metrics/processes` accepts `?type=memory` query parameter. SSE stream only sends partial metrics. None of this is documented.

### DOC-12: "Compression Bypass Fix" Mentioned Without Explanation

**File:** `README.md:117`
**Severity:** SUGGESTION

References the `ContentEncoding::Identity` fix without explaining why compression breaks SSE.

---

## Prioritized Fix Plan

### Phase 1: Critical Fixes (Immediate)

| # | Finding | Task | Est. Complexity |
|---|---------|------|-----------------|
| 1 | CRIT-02 | Replace `unwrap()` with poison recovery on mutex lock | Simple (1 line) |
| 2 | CRIT-03 | Add zero-check in memory collector (match disk pattern) | Simple (3 lines) |
| 3 | CRIT-05 | Fix SSE reconnect timer reset (`reconnectTimer = null`) | Simple (1 line) |
| 4 | SEC-03 | Fix UB in config test (`set_var`/`remove_var`) | Medium |
| 5 | DOC-01 | Resolve version mismatch (Cargo.toml vs README) | Simple |

### Phase 2: Architecture Redesign (High Impact)

| # | Finding | Task | Est. Complexity |
|---|---------|------|-----------------|
| 6 | CRIT-01, ARCH-02 | Background collection task with `watch` channel | High |
| 7 | CRIT-04, DATA-01 | Move Networks/Disks into global state, compute rates | Medium |
| 8 | ARCH-01 | Wire Config through to handlers via `web::Data` | Medium |
| 9 | ARCH-03 | Remove redundant process refresh from collector | Simple |

### Phase 3: Security & Reliability

| # | Finding | Task | Est. Complexity |
|---|---------|------|-----------------|
| 10 | SEC-01 | Remove wildcard CORS header | Simple |
| 11 | SEC-02 | Replace hand-rolled html_escape with tested crate | Simple |
| 12 | UI-01 | Remove/fix debug utilities in production | Simple |
| 13 | UI-02 | Fix dual reconnection mechanism | Medium |
| 14 | UI-04,05 | Remove unused SSE extension, or add SRI | Simple |

### Phase 4: Testing & Build

| # | Finding | Task | Est. Complexity |
|---|---------|------|-----------------|
| 15 | BUILD-01 | Create CI/CD pipeline | Medium |
| 16 | BUILD-02 | Add integration tests (html_escape, format_sse_update, routes) | High |
| 17 | BUILD-03 | Replace once_cell with std::sync::LazyLock | Simple |
| 18 | BUILD-04 | Narrow tokio features | Simple |
| 19 | BUILD-05 | Add [profile.release] configuration | Simple |
| 20 | BUILD-07 | Rename misnamed test function | Simple |

### Phase 5: UI/UX & Accessibility

| # | Finding | Task | Est. Complexity |
|---|---------|------|-----------------|
| 21 | UI-06 | Add stale data indication after disconnect | Medium |
| 22 | UI-07 | Add HTMX error event handling | Simple |
| 23 | UI-09 | Move inline styles to CSS classes | Medium |
| 24 | A11Y-01 | Add ARIA attributes to progress bars | Simple |
| 25 | A11Y-02 | Add aria-hidden to emoji icons | Simple |
| 26 | A11Y-03 | Add semantic HTML landmarks | Simple |

### Phase 6: Documentation & Polish

| # | Finding | Task | Est. Complexity |
|---|---------|------|-----------------|
| 27 | DOC-02-12 | Update README to match actual code behaviour | Medium |
| 28 | DATA-02 | Separate CPU/memory filters in process collector | Simple |
| 29 | DATA-04 | Rename core_count or add physical_core_count | Simple |
| 30 | UI-10 | Remove console.log from production | Simple |

---

## Full Findings Index

| ID | Severity | Category | File(s) | Description |
|----|----------|----------|---------|-------------|
| CRIT-01 | CRITICAL | Core | system.rs:48,79 | Global mutex blocks async runtime with sleeps |
| CRIT-02 | CRITICAL | Core | system.rs:48 | Mutex poison panic crashes server |
| CRIT-03 | CRITICAL | Data | memory.rs:22 | Division by zero produces NaN |
| CRIT-04 | CRITICAL | Data | network.rs:13, disk.rs:13 | New sysinfo instances yield stale data |
| CRIT-05 | CRITICAL | UI | dashboard.html:491-501 | SSE reconnection stops after first retry |
| ARCH-01 | ISSUE | Core | config, metrics.rs, process.rs | Config values parsed but unused |
| ARCH-02 | ISSUE | Core | routes/metrics.rs | Redundant full collection per request |
| ARCH-03 | ISSUE | Core | system.rs, process.rs | Triple process refresh per cycle |
| ARCH-04 | WARNING | Core | system.rs:23-41 | Lazy init blocks first request |
| ARCH-05 | WARNING | Core | main.rs:57 | Relative static file path |
| ARCH-06 | WARNING | Core | config/mod.rs, main.rs | Logging before logger init |
| DATA-01 | ISSUE | Data | network.rs:20-21 | Cumulative totals, not throughput |
| DATA-02 | WARNING | Data | process.rs:33-37 | CPU-intensive low-memory processes hidden |
| DATA-03 | WARNING | Data | process.rs:75-98 | Aggressive memory deduplication |
| DATA-04 | WARNING | Data | cpu.rs:26 | core_count is logical, not physical |
| DATA-05 | WARNING | Data | cpu.rs:17-21 | Frequency from first core only |
| DATA-06 | WARNING | Data | disk.rs:18-25 | Usage includes reserved blocks |
| DATA-07 | SUGGESTION | Data | system.rs:94 | Timestamp lacks timezone |
| DATA-08 | SUGGESTION | Data | All collectors | No percentage clamping |
| DATA-09 | SUGGESTION | Data | Models | Mixed f32/f64 precision |
| DATA-10 | SUGGESTION | Data | memory model | Missing swap_usage_percent |
| DATA-11 | ISSUE | Data | system.rs:76-86 | Inverted sleep condition |
| UI-01 | BUG | UI | dashboard.html:550-558 | Debug raw() breaks dashboard |
| UI-02 | WARNING | UI | dashboard.html:464+ | Dual reconnection mechanisms |
| UI-03 | WARNING | UI | dashboard.html:497 | No exponential backoff |
| UI-04 | WARNING | UI | dashboard.html:14 | SSE extension loaded, unused |
| UI-05 | WARNING | UI | dashboard.html:14 | CDN script lacks SRI |
| UI-06 | ISSUE | UI | dashboard.html | No stale data indication |
| UI-07 | WARNING | UI | dashboard.html:268+ | HTMX polling no error handling |
| UI-08 | ISSUE | UI | dashboard.html:238,254 | CSS width missing unit |
| UI-09 | WARNING | UI | metrics.rs:147-213 | Inline styles in HTML fragments |
| UI-10 | SUGGESTION | UI | dashboard.html | Console logging in production |
| UI-11 | SUGGESTION | UI | metrics.rs:247-257 | html_escape per-char allocation |
| UI-12 | WARNING | UI | models, metrics.rs | Collected data never displayed |
| UI-13 | SUGGESTION | UI | metrics.rs:147-154 | Process name overflow |
| A11Y-01 | ISSUE | A11Y | dashboard.html:237+ | No ARIA on progress bars |
| A11Y-02 | ISSUE | A11Y | dashboard.html:234+ | Emoji icons confuse screen readers |
| A11Y-03 | WARNING | A11Y | dashboard.html | No semantic landmarks |
| SEC-01 | ISSUE | Security | metrics.rs:67 | Wildcard CORS on SSE |
| SEC-02 | WARNING | Security | metrics.rs:247-257 | Hand-rolled html_escape |
| SEC-03 | BUG | Security | config/mod.rs:127-130 | UB in tests (set_var) |
| BUILD-01 | ISSUE | Build | Project root | No CI/CD |
| BUILD-02 | ISSUE | Build | routes/, services/ | Zero HTTP test coverage |
| BUILD-03 | ISSUE | Build | Cargo.toml, system.rs | Unnecessary once_cell |
| BUILD-04 | WARNING | Build | Cargo.toml:9 | Overly broad tokio features |
| BUILD-05 | ISSUE | Build | Cargo.toml | No release profile |
| BUILD-06 | ISSUE | Build | Cargo.toml:31-32 | Empty dev-dependencies |
| BUILD-07 | WARNING | Build | network.rs:44 | Misnamed test function |
| BUILD-08 | SUGGESTION | Build | Project root | No rustfmt.toml/clippy.toml |
| BUILD-09 | SUGGESTION | Build | Project root | No rust-toolchain.toml |
| BUILD-10 | WARNING | Build | utils/mod.rs:13 | Redundant #[allow(dead_code)] |
| BUILD-11 | SUGGESTION | Build | Project root | No benchmarks |
| DOC-01 | BUG | Docs | Cargo.toml, README | Version mismatch |
| DOC-02 | ISSUE | Docs | README | "Zero JS Dependencies" false |
| DOC-03 | WARNING | Docs | README, config | Missing trace log level |
| DOC-04 | ISSUE | Docs | README | Placeholder repo URLs |
| DOC-05 | WARNING | Docs | README, metrics.rs | Config vs hardcoded interval |
| DOC-06 | WARNING | Docs | README, metrics.rs | "All filesystems" vs .take(3) |
| DOC-07 | WARNING | Docs | system.rs:112 | Incomplete comment |
| DOC-08 | WARNING | Docs | README, system.rs | "Production Ready" vs unwrap() |
| DOC-09 | SUGGESTION | Docs | Cargo.toml | Missing metadata |
| DOC-10 | SUGGESTION | Docs | Project root | No CHANGELOG |
| DOC-11 | SUGGESTION | Docs | README | API docs incomplete |
| DOC-12 | SUGGESTION | Docs | README | Compression bypass unexplained |

---

*Audit conducted by parallel analysis agents across 5 domains. No code changes were made during this audit.*
