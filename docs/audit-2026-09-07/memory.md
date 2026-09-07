# Memory, handle and unbounded-growth audit

Date: 2026-09-07
Scope: `src-tauri/src/` (Rust) and `src/` (TypeScript/React), read-only.
Method: code reading, plus arithmetic on the real sample format, plus measurements taken
from this machine's own `%LOCALAPPDATA%\Lalia\logs` and `%APPDATA%\Lalia\lalia.db`.
Nothing was built, run or modified.

One caveat on freshness: `src-tauri/src/logging.rs` was rewritten by another session at
17:24:11 while this audit was in progress, which resolved H-3. Every other cited file was
last modified on 2026-09-06 or earlier and was read in its current state. Line numbers were
re-verified against disk after the report was drafted.

The GPU VRAM eviction that degraded latency from about 1 s to about 30 s is treated as
already root-caused and is out of scope. This report covers host RAM, OS handles, threads
and collections that only grow.

---

## Worst offender for an app that runs all day

**The `whisper-server.exe` child process is never recycled while it is healthy.** That is
the 1843 MB you measured after 9 hours, and it is host RAM, so it belongs in this report
even though its allocator lives inside whisper.cpp.

The app has exactly four ways to restart the engine:

| Trigger | Where |
|---|---|
| The child is **dead** | `src-tauri/src/engine.rs:191-204` |
| A transcription **timed out** | `src-tauri/src/pipeline.rs:685-691` |
| Settings changed / model changed | `src-tauri/src/engine.rs:74` |
| The user clicks Restart engine | `src-tauri/src/commands.rs` (`engine_restart`) |

The watchdog at `src-tauri/src/engine.rs:196-199` reads:

```rust
let dead = match local {
    Some(s) => s.info().status == EngineStatus::Ready && !s.is_alive(),
    None => false,
};
```

`is_alive()` is `try_wait()` on the child handle (`src-tauri/src/asr/whisper_server.rs:250-256`),
so the only condition that restarts a running engine is that it stopped being a running
engine. There is no age limit, no idle limit and no resident-set limit anywhere in the tree.
A healthy child that has served a day of dictations is never replaced.

**CONFIRMED (traced):** no recycle policy exists, and the app feeds this child an unusually
wide spread of request sizes. Segments dispatched mid-speech are at least 2.5 s
(`SEGMENT_MIN_MS`, `src-tauri/src/pipeline.rs:93`) and at most 12 s (`SEGMENT_MAX_MS`,
line 96), while a final pass can be the whole recording, up to 600 s
(`src-tauri/src/settings.rs:108`). That is a 240x spread of buffer sizes hitting one
long-lived allocator.

**SUSPECTED (not verified):** that the 1843 MB is allocator fragmentation and per-request
state retention inside whisper.cpp driven by that spread. Confirming this needs the child
profiled while running, which this audit did not do. What is certain is that the app gives
whatever it accumulates an unbounded amount of time to accumulate.

**Fix.** Add an idle recycle to the existing 5 s watchdog loop in `src-tauri/src/app.rs:194-207`:
when `snapshot.lock().phase == Phase::Idle` has held for N minutes (15 is a reasonable
start) and the engine has been up longer than M minutes (60), call `engine.apply(...)`.
The cost is already measured and logged for you: `warm_ms` at
`src-tauri/src/asr/whisper_server.rs:212-219`. Optionally gate it on the child's working
set via `GetProcessMemoryInfo` so a well-behaved build is never restarted.

---

## Two premises in the brief are wrong

Both matter, because acting on them would send the fix in the wrong direction.

### The keyboard hook does unhook the old one, and the timer is 30 seconds

`src-tauri/src/hotkey.rs:500-512`:

```rust
if msg.message == WM_TIMER {
    match SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), hmod, 0) {
        Ok(fresh) => {
            let _ = UnhookWindowsHookEx(hook);   // line 503
            hook = fresh;
            rehooks += 1;
            if rehooks % 20 == 1 {               // line 506
                tracing::debug!("keyboard hook re-registered ({rehooks} times so far)");
            }
        }
```

Line 503 unhooks the old hook immediately after the new one is installed. **There is no
accumulating HHOOK leak.** The timer is `SetTimer(None, 1, 30_000, None)` at line 497, so
30 seconds, and the log prints only every 20th re-registration, so one line every 10 minutes.
That is where the "roughly every 10 minutes" impression comes from.

The arithmetic confirms it exactly. From this machine's log:

```
13:44:47  keyboard hook re-registered (1081 times so far)
13:54:47  keyboard hook re-registered (1101 times so far)
```

20 re-registrations in 600 s is one per 30 s. And 1081 x 30 s = 32,430 s = **9 hours 0 minutes**,
which matches "the app had been running about 9 hours" to the minute. The counter is a clock,
and what it reports is 9 hours of uptime. No resource is accumulating behind it.

Residual risks are real but small, and are filed as **L-2** below.

### The stored audio is 16 kHz; the 48 kHz in the log is the device format

`src-tauri/src/audio.rs:18` sets `TARGET_RATE = 16_000`. The 48000 Hz / 1 ch / F32 line in
your log comes from `src-tauri/src/audio.rs:321`, which reports the **device** format. Every
callback is downmixed and resampled to 16 kHz before it reaches storage
(`src-tauri/src/audio.rs:340-344`). So the growth rate is:

| Quantity | Value |
|---|---|
| Stored format | 16,000 Hz, mono, `f32` (4 bytes) |
| Growth rate | 64 KB/s = **3.84 MB/min** = **230 MB/hour** |
| 10-minute dictation | 9,600,000 samples x 4 B = **38.4 MB** |
| 1-hour dictation | **230.4 MB** |
| Counterfactual: if raw 48 kHz f32 were stored | 192 KB/s = **691 MB/hour** |

The 1-hour figure cannot be reached with default settings, because the buffer is capped
(see the next section) and the level task force-stops the recording at the cap
(`src-tauri/src/pipeline.rs:419-421`). It becomes reachable the moment a user raises
`max_recording_seconds` in settings.

---

## Severity summary

| Severity | Count |
|---|---|
| CRITICAL | 0 |
| HIGH | 3 |
| MEDIUM | 6 |
| LOW | 6 |

A fourth HIGH, H-3, was fixed on disk at 17:24 during this audit and is kept below as a
record with its measurements. It is excluded from the count above.

There is no CRITICAL. Nothing in the host process grows without a bound until it exhausts
memory: the sample buffer is capped, every collection is either per-session or replaced
wholesale, and the hook does not leak handles. The honest shape of the problem is a large
child process that is never recycled, plus a set of cleanup routines that only ever run at
startup.

---

# HIGH

## H-1. The speech engine child is never recycled

**Where:** `src-tauri/src/engine.rs:191-204` (the only health check),
`src-tauri/src/app.rs:194-207` (the 5 s watchdog loop that could carry the fix).

**Mechanism:** covered in full in the "Worst offender" section above. The watchdog restarts
the child only when `try_wait()` says it has exited. Health, age and memory are never
considered.

**Impact:** measured at 1843 MB resident after about 9 hours on your machine. The app
itself is not holding that memory, and it also never asks the OS to reclaim it.

**Fix:** idle-triggered recycle in the existing watchdog. See the "Worst offender" section
for the exact placement and the already-measured warm cost.

---

## H-2. History retention and database backup pruning run only at startup, so an app left open never prunes either

**Where:**

| Cleanup | Defined at | Called from | Runs when |
|---|---|---|---|
| History retention | `src-tauri/src/db.rs:423-430` | `src-tauri/src/app.rs:144-150` | **Startup only** |
| Database backup prune (14 days) | `src-tauri/src/db.rs:256-266` | `src-tauri/src/db.rs:277-281`, inside `Db::open` | **Startup only** |
| Event journal prune (7 days) | `src-tauri/src/journal.rs:104-120` | `src-tauri/src/journal.rs:94` | Day rollover (acceptable) |
| Text log prune (7 days) | `src-tauri/src/logging.rs:86-101` | `src-tauri/src/logging.rs:65` | Day rollover (acceptable) |

The two day-rollover entries are fine: they prune at the moment they roll, so a
continuously running app does clean them. The two marked **Startup only** are the finding.

**Mechanism:** this is a background tray app. `src-tauri/src/lib.rs:47-55` intercepts
`CloseRequested` on the main window and hides it, so closing the dashboard does not restart
the process. A user who leaves it running for three weeks calls `Db::open` once, in week
one. Retention and backup pruning simply never happen again.

The default makes it worse: `retention_days` is `None`
(`src-tauri/src/settings.rs:244` and `256-263`, "None = keep forever"), and
`keep_history` is `true`. So on a default install `delete_history_older_than` is never
called at all, at startup or otherwise.

**Impact, measured and projected.** From this machine's database, copied and inspected
read-only:

- 152 history rows written between 2026-09-04T22:01 and 2026-09-07T14:13, which is
  64.2 hours, so **56.7 rows/day**. The text logs agree: 64, 59 and 38 "recording started"
  lines on 09-05, 09-06 and 09-07.
- Average text payload per row (`raw_text` + `cleaned_text` + `final_text`): **1,058 bytes**.
- Whole file: 475,136 bytes for 152 rows, so about **3.1 KB per row** once indexes and page
  overhead are counted.

Projected for this user over one year:

| | Year 1 |
|---|---|
| History rows | ~20,700 |
| Text stored | ~27 MB |
| `lalia.db` on disk | **~65 MB** |
| `backup/` directory (14 daily full copies, see M-6) | **~900 MB** |

**Fix:** move retention and backup pruning onto a periodic tick. The 5 s watchdog loop at
`src-tauri/src/app.rs:194-207` already exists; run these once every 6 hours from a
counter there, or add a dedicated daily task. Separately, reconsider `retention_days: None`
as a shipped default for a public app.

---

## H-3. RESOLVED DURING THIS AUDIT: the text log was never pruned

**Status: fixed on disk at 2026-09-07 17:24:11, by a concurrent edit, while this audit was
being written.** Reported here because the finding was real, the evidence is worth keeping,
and the log files written before the fix are still on disk.

**What it was.** `src-tauri/src/logging.rs:15` used
`tracing_appender::rolling::daily(&dir, "lalia.log")` with no `.max_log_files(n)`, and that
roller keeps every file forever. Nothing else in the tree deleted `lalia.log.*`: the journal
prune at `src-tauri/src/journal.rs:104-120` filters on
`starts_with("events-") && ends_with(".jsonl")`, so it skipped the text logs by construction.

**Measured before the fix,** and still on disk now:

```
15,307  lalia.log.2026-09-04
318,137 lalia.log.2026-09-05
604,189 lalia.log.2026-09-06
308,429 lalia.log.2026-09-07   (partial day, up to 17:14)
```

At about 450 KB/day that projected to roughly 164 MB/year, growing for the life of the
install.

**What replaced it.** `logging.rs` now ships a custom `LocalDaily` writer
(`src-tauri/src/logging.rs:38-82`) that rolls at **local** midnight, plus a `prune`
(`src-tauri/src/logging.rs:86-101`) that keeps `KEEP_DAYS = 7` (line 22), called from
`file_for_today` at line 65 when the day rolls over.

**Verified as sound.** This is a day-boundary trigger, which is the pattern H-2 criticises,
but here it is correct: the roll and the prune are the same event, so the first log line
written after local midnight prunes. A continuously running app therefore does prune, which
is exactly what H-2's startup-only routines fail to do. The prune filter is
`n.starts_with(PREFIX)` where `PREFIX = "lalia.log."` (line 20, 91), so it leaves
`events-*.jsonl` alone, and the accompanying test at lines 156-180 asserts that.

**Still outstanding:** the four files above predate the fix and sit under the 7-file limit,
so none has been pruned yet. More importantly the **volume** is untouched, and it is driven
by L-1 below. Fixing L-1 remains worthwhile.

---

## H-4. A COM apartment is initialized once per dictation and never uninitialized

**Where:** `src-tauri/src/context.rs:162-191`

```rust
pub fn inspect_focus(read_text: bool) -> FocusInfo {
    let handle = std::thread::spawn(move || unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);   // line 164
        let Ok(automation): windows::core::Result<IUIAutomation> = CoCreateInstance(...) else { ... };
        ...
    });
    handle.join().unwrap_or_default()
}
```

**Mechanism:** a fresh OS thread is spawned for each call, `CoInitializeEx` is called on it,
and the thread then exits. `CoUninitialize` is **never called**, anywhere in the codebase.
Verified: `grep -rn "CoUninitialize" src-tauri/src/` returns nothing, and
`CoInitializeEx` appears exactly once, at `src-tauri/src/context.rs:164`. Every
`CoInitializeEx` must be balanced by a `CoUninitialize` on the same thread; a thread that
exits without it leaves its apartment reference outstanding.

The call site is not conditional. `src-tauri/src/pipeline.rs:372-382` calls it on every
`begin()`, which is every dictation start, including starts that are then refused:

```rust
let info = match tokio::time::timeout(Duration::from_millis(400),
    tokio::task::spawn_blocking(move || crate::context::uia::inspect_focus(read_ctx))).await
```

Note that `read_ctx` only controls whether nearby text is read. The password-field check
runs regardless of the Context Awareness setting, so this fires for every user on default
settings.

**Impact:** at this machine's measured **57 dictations/day**, that is 57 unbalanced
`CoInitializeEx` calls per day and about **20,800 per year** in one process. I could not
measure the per-occurrence cost without running the app, so I will not invent a number for
it. What is certain from the code is that the imbalance exists and repeats once per
dictation forever.

**Secondary mechanism (SUSPECTED, unobserved here).** The 400 ms `tokio::time::timeout`
protects pipeline latency and does nothing else. `spawn_blocking` tasks cannot be cancelled,
and the inner `std::thread::spawn` plus `handle.join()` at line 190 is a hard block. When
UI Automation is slow, which the comment at `src-tauri/src/pipeline.rs:370-371` says happens
for "seconds" in Chromium and Electron apps, the timeout returns to the pipeline while two
threads stay blocked: one tokio blocking-pool thread and one UIA thread holding a live
`IUIAutomation`. They do finish eventually, so this is a transient pile-up and not a
permanent leak, but Tokio's blocking pool caps at 512 threads by default and the paste path
also depends on `spawn_blocking` (`src-tauri/src/pipeline.rs:765`). I checked the logs for
evidence: `grep -c "UI Automation did not answer"` returns **0 across all four days**, so
this has never fired on your machine. Treat it as a latent risk on slower machines.

**Fix:** call `CoUninitialize()` at the end of the closure, after the `FocusInfo` is built
and the `IUIAutomation` has been dropped. Better still, keep one dedicated long-lived UIA
thread with a channel, which removes both the per-dictation thread creation and the
imbalance in one change.

---

# MEDIUM

## M-1. A failed warm-up leaves the child running, and the CPU fallback reuses its port

**Where:** `src-tauri/src/asr/whisper_server.rs:226-229` and `src-tauri/src/engine.rs:127-138`.

**Mechanism, traced.** `WhisperServer::start()` has several failure exits. Three of them are
clean: exe or model missing (no child spawned), spawn failure (no child), and the 120 s
timeout, which calls `self.stop().await` at line 184. The fourth is not:

```rust
Err(e) => {
    self.set(EngineStatus::Failed, Some(format!("warm-up failed: {e}")));
    Err(e)          // line 228, no stop()
}
```

The child has already spawned, bound its port and loaded the model at this point. It is left
running.

`engine.rs` then builds the CPU fallback from the failed server's own config:

```rust
let mut cfg = server.config().clone();   // line 129, carries `port` unchanged
cfg.use_gpu = false;
let cpu = Arc::new(WhisperServer::new(cfg));
*self.local.write() = Some(cpu.clone());  // line 132, replaces without stopping the old
result = cpu.start().await;
```

`pick_port()` (`src-tauri/src/engine.rs:62-71`) caches the port for the process lifetime, so
the clone carries the same port. The new CPU child then cannot bind it, while the readiness
probe in `start()` is a bare TCP connect:

```rust
if tokio::net::TcpStream::connect(("127.0.0.1", self.cfg.port)).await.is_ok() {
    break;                              // whisper_server.rs:201
}
```

which the still-live GPU process satisfies immediately. The app can therefore believe the
CPU engine came up while it is in fact talking to the GPU process it thinks it abandoned.

**Can orphans accumulate?** No, and this is worth stating plainly. Two independent
safety nets catch the abandoned child:

1. `cmd.kill_on_drop(true)` at `src-tauri/src/asr/whisper_server.rs:134`. The GPU `server`
   Arc stays in scope until the end of `apply()`, and its `Drop` kills the child then.
2. The job object, see the "checked negatives" section.

So the failure mode is limited to a **temporary two-process window** and a confused
engine state. Orphans do not accumulate.

**Evidence from your logs.** The CPU fallback has fired: six times on 2026-09-04
(`grep -n "GPU start failed"`). Every one of those was the safe path: the child had already
exited with `error: unknown argument: --no-context`, which produced
`engine start failed: engine not ready: process exited`. Two later entries at 21:50:59 and
21:54:10 failed with `error sending request for url (http://127.0.0.1:63707/inference)`,
which is consistent with the port-reuse path but is not proof of it, since that build was
broken in other ways. **The warm-up-failure path itself has not been observed firing on
this machine.** That is why this is rated MEDIUM.

**Fix:** two lines. Call `self.stop().await` before returning the warm-up error at
`whisper_server.rs:227`, and reset the cached port (or pick a fresh one) when building the
CPU fallback config in `engine.rs:129`. Consider also making the readiness probe an HTTP
request to a known endpoint so it cannot be satisfied by a stale listener.

---

## M-2. The last dictation's samples are kept in memory whenever it did not end in success

**Where:** `src-tauri/src/pipeline.rs:209`, `572`, `811-814`.

**Mechanism:** the recovery slot is declared for the life of the pipeline task:

```rust
let last_recovery: Arc<Mutex<Option<(Vec<f32>, AppContext)>>> = Arc::new(Mutex::new(None));
```

and filled with a full clone before transcription:

```rust
*last_recovery.lock() = Some((speech.clone(), ctx.clone()));   // line 572
```

It is cleared only on the happy path:

```rust
if status == "success" || status == "copied" {
    *last_recovery.lock() = None;                               // line 812
```

Four common paths return before ever reaching line 812 and therefore leave the samples
resident: the engine is not ready (line 594-599), transcription failed (line 674-684),
transcription timed out (line 685-694), and the transcript was empty or a known
hallucination (line 710-714). That last one is the one that matters, because a
long recording that whisper returns nothing useful for is exactly the case where the
retained buffer is largest.

**Impact:** up to **38.4 MB** resident until the next successful dictation replaces or
clears it, at default settings. The slot is bounded and each dictation overwrites it, so this is a retention with a
ceiling. Note also that line 572 is a `clone()` of a buffer
that is already a copy (`analyze_speech` returns `samples[start..end].to_vec()` at
`src-tauri/src/audio.rs:528`), so the peak while processing a 10-minute dictation is
roughly `samples` 38.4 MB + `speech` 38.4 MB + the recovery clone 38.4 MB + the encoded
WAV 19.2 MB = **about 134 MB transient**.

**Fix:** store `Arc<Vec<f32>>` to drop the clone, and clear the slot on every terminal
path, including the four listed above. A time-based expiry (drop the recovery buffer
after a few minutes) would also cap it.

---

## M-3. The recording buffer reallocates by doubling inside the realtime audio callback

**Where:** `src-tauri/src/audio.rs:58-61` and the callback at `src-tauri/src/audio.rs:340-344`.

```rust
if self.recording {
    let remaining = self.max_samples.saturating_sub(self.buffer.len());
    self.buffer.extend_from_slice(&samples[..samples.len().min(remaining)]);
```

**Mechanism:** `buffer` starts empty. `stop_recording` uses `std::mem::take`
(`src-tauri/src/audio.rs:173`), which moves the Vec out and leaves capacity 0, so every new
recording grows from zero. `extend_from_slice` with no reserved capacity grows by doubling,
so reaching the 38.4 MB cap costs about 22 reallocations and roughly 76 MB of total memcpy,
with the final copy alone moving about 19 MB.

That work happens on the cpal callback thread, holding the lock:

```rust
move |data: &[f32], _| {
    let mono = to_mono(data, channels);
    let out = resampler.process(&mono);
    rec.lock().push(&out);          // line 343
},
```

At 48 kHz with typical 10 ms buffers the callback budget is about 10 ms. A 19 MB memcpy
consumes a meaningful slice of that. The code already handles the symptom: `ErrorKind::Xrun`
is caught and logged at `src-tauri/src/audio.rs:325`.

`to_mono` also allocates a fresh `Vec` per callback (`data.to_vec()` at line 382), as does
`Resampler::process`, so there are roughly 100 short-lived allocations per second in the
realtime path.

**Impact:** dropped audio (a lost syllable) during long dictations, plus steady allocator
churn all day. No memory is retained; this is a realtime-safety and audio-quality issue.

**Fix:** allocate once. In `AudioCapture::new` and `configure`, call
`buffer.reserve(max_samples)` (or build with `Vec::with_capacity`) so the recording path
never reallocates. For the per-callback allocations, hold reusable scratch buffers in the
`Resampler` and the callback closure.

---

## M-4. `discard()` keeps the buffer's full capacity forever

**Where:** `src-tauri/src/audio.rs:204-209`

```rust
pub fn discard(&self) {
    let mut r = self.rec.lock();
    r.recording = false;
    r.buffer.clear();      // line 207: keeps capacity
    r.preroll.clear();
}
```

**Mechanism:** `Vec::clear()` sets the length to zero and keeps the allocation. Compare
`stop_recording` at line 173, which correctly uses `std::mem::take` and releases it. The two
callers of `discard()` are the cancel path (`src-tauri/src/pipeline.rs:462`, Escape pressed)
and the password-field refusal (`src-tauri/src/pipeline.rs:384`).

**Impact:** cancel a 10-minute dictation with Escape and the process holds **38.4 MB** of
idle capacity until the next `stop_recording` takes it. The `start_recording` path at line
162 also uses `clear()`, so that capacity survives subsequent recordings too.

**Fix:** replace `r.buffer.clear()` with `r.buffer = Vec::new()` (or `shrink_to_fit()`
after clearing). If M-3 is fixed by pre-reserving the cap, then make `discard` shrink back
to that reserved size, so the two paths stay consistent.

---

## M-5. A snapshot of the user's clipboard can be held indefinitely

**Where:** `src-tauri/src/insertion.rs:300`, `320`, `350`, `668-690`, `700-703`.

**Mechanism:** before each paste, the clipboard thread reads and stores whatever was there:

```rust
let saved = read_clipboard_text();
...
sh.saved_text = saved;          // line 320
```

`saved_text` is emptied in exactly one place, the restore handler:

```rust
WM_LALIA_RESTORE => {
    let saved = st.shared.lock().saved_text.take();    // line 350
```

and `WM_LALIA_RESTORE` is posted from exactly one place:

```rust
if opts.restore_clipboard {
    let _ = PostMessageW(Some(hwnd()), WM_LALIA_RESTORE, ...);   // line 702
```

Two routes therefore skip it. First, `restore_clipboard: false` in settings
(`src-tauri/src/settings.rs:191`). Second, and more common, the "paste not consumed" path at
line 668-690 returns early, before line 700 is ever reached.

`read_clipboard_text` caps its scan at 50,000,000 UTF-16 units
(`src-tauri/src/insertion.rs:239`), so the ceiling is about **100 MB**.

**Impact:** a single retained snapshot, overwritten on the next successful paste, so it does
not accumulate. It is still up to 100 MB of the user's clipboard held by a background
process for an unbounded time, which is a privacy consideration as much as a memory one.

**Fix:** clear `saved_text` on the early return at line 683, and whenever
`opts.restore_clipboard` is false, right after the publish completes.

---

## M-6. The database backup keeps 14 full copies

**Where:** `src-tauri/src/db.rs:243-268`, called from `src-tauri/src/db.rs:277-281`.

**Mechanism:** `daily_backup` copies the whole `lalia.db` to `backup/lalia-YYYY-MM-DD.db`
and deletes copies older than 14 days. The comment at line 276 says "Cheap: the file is a
few hundred KB", which is true today (measured 475,136 bytes) and stops being true as
history accumulates under H-2's missing retention.

**Impact:** the backup directory is always about 14x the database size. With H-2's projected
65 MB database at the end of year one, that is about **900 MB** of backups. The two
mechanisms compound.

Also note this only runs inside `Db::open`, so a long-running session takes one backup and
then none, which is the H-2 pattern again.

**Fix:** keep fewer copies as the file grows (for example 7 daily plus 3 monthly), or
compress them. Fixing H-2's retention caps this automatically.

---

# LOW

## L-1. Every modifier keystroke is written to the log at DEBUG, in release builds

**Where:** `src-tauri/src/logging.rs:110` and `src-tauri/src/hotkey.rs:342-350`.

```rust
let filter = EnvFilter::try_from_env("LALIA_LOG").unwrap_or_else(|_| EnvFilter::new("info,lalia_lib=debug"));
```

`lalia_lib=debug` applies unconditionally, with no `#[cfg(debug_assertions)]` guard, so
release builds log at DEBUG too. This line survived the 17:24 logging rewrite unchanged. The hook callback then logs on both edges of every
Ctrl, Alt and Win key:

```rust
if matches!(vk, VK_LMENU | VK_RMENU | VK_LCONTROL | VK_RCONTROL | VK_LWIN | VK_RWIN) {
    tracing::debug!("key {} {} flags={:#x} injected={}", ...);
}
```

No key identity beyond modifiers is recorded, so this is not a privacy problem. It is the
main driver of the log volume measured in H-3, and it puts a formatting call plus a channel
send on the low-level hook path, which Windows silently removes if it ever exceeds
`LowLevelHooksTimeout`. That is the exact failure the 30 s re-registration exists to paper over.

**Fix:** default the filter to `info,lalia_lib=info` in release and keep `debug` behind
`debug_assertions` or the existing `debug_mode` setting. `LALIA_LOG` already provides the
escape hatch for diagnosis.

## L-2. The hook re-registration is a 2,880-per-day workaround with three small rough edges

**Where:** `src-tauri/src/hotkey.rs:490-512`.

The mechanism is sound (see the premises section, the old hook is unhooked). Three residual points:

1. Line 503, `let _ = UnhookWindowsHookEx(hook)`. The result is discarded, so a failure to
   unhook would be silent and would genuinely leak, with nothing in the log to show it.
2. Between line 501 and line 503 both hooks are installed. A keystroke landing in that
   window runs `hook_proc` twice, mutating `st.down` and potentially emitting a duplicate
   `Pressed`/`Released`. The window is microseconds wide and 2,880 of them occur per day.
3. The comment at lines 490-494 describes the condition being guarded as a boot-time one
   ("right after a reboot while the binary's pages are still on disk"). Re-registering every
   30 s forever to cover the first minute after boot is a heavy remedy.

**Fix:** log a warning when `UnhookWindowsHookEx` fails. Back the timer off after the first
few minutes of uptime (30 s for the first 5 minutes, then 5 minutes thereafter), which cuts
this from 2,880 to about 300 cycles per day and shrinks the double-hook exposure with it.

## L-3. Abandoned segment jobs when one segment fails

**Where:** `src-tauri/src/pipeline.rs:618-639`.

`std::mem::take(&mut g.jobs)` moves every outstanding `JoinHandle` into a local, then the
loop `break`s on the first failure (line 634-637). The remaining handles are dropped, and a
dropped Tokio `JoinHandle` detaches the task and lets it run to completion. Those tasks keep
running and keep hitting the engine, at the same moment the code falls back to transcribing
the whole recording in one pass. Compare the cancel path at line 459-461, which correctly
calls `.abort()` on each.

**Fix:** `for j in jobs { j.abort(); }` before breaking, mirroring `cancel`.

## L-4. `keep_audio` WAV files have no cleanup outside startup retention

**Where:** `src-tauri/src/pipeline.rs:818-828`.

Default is off (`keep_audio: false`, `src-tauri/src/settings.rs:258`). When a user turns it
on, each dictation writes `local_dir()/audio/{uuid}.wav`, which is about 19.2 MB for a
10-minute dictation (16-bit WAV, half the f32 size). The only unlink paths are history
deletion (`src-tauri/src/db.rs:408-430` returning paths for the caller to remove), which
H-2 shows runs at startup only and never at all on default settings.

At this machine's 57 dictations/day, even 30-second average clips would be about 55 MB/day,
or **20 GB/year**, with no automatic cleanup.

**Fix:** covered by making retention periodic (H-2). Consider also a size cap on the audio
directory, independent of row retention.

## L-5. The hidden dashboard polls the backend every 2 seconds forever

**Where:** `src/App.tsx:46`, with `src-tauri/src/lib.rs:47-55`.

```tsx
const iv = setInterval(() => api.snapshot().then(setSnap).catch(() => {}), 2000);
```

The `useEffect` cleanup is correct (line 47-52 clears the interval and unlistens both Tauri
subscriptions), so this is not a leak. But closing the dashboard hides the window and keeps it
alive (`lib.rs:47-55`), so the WebView survives and the interval keeps running:
**43,200 IPC round-trips per day** for a window nobody is looking at. Each one takes the
audio lock via `set_phase` -> `shared.audio.is_open()`.

**Fix:** pause on `document.visibilityState === "hidden"`, or drive the snapshot purely from
the `lalia://overlay` event that already exists at line 45 and drop the polling entirely.

## L-6. Small collections that only grow

- **`learning_events`** (`src-tauri/src/db.rs:79-86`) stores `before_text` and `after_text`
  for every classified edit and has no cap or age-based pruning. The only delete is the
  user-triggered `delete_learning_data` (`src-tauri/src/db.rs:603`). Volume is low: 14 rows
  on this machine.
- **`suggestions`** is upserted by (kind, wrong, correct) so it is bounded by distinct
  corrections. 11 rows here.
- **`src/App.tsx:71`**, `setTimeout(() => setToastMsg(null), 2600)` is never cleared, so a
  rapid sequence of toasts leaves overlapping timers. Harmless, worth tidying.

---

# Checked and found fine

Stating these explicitly, because three of them were the brief's leading hypotheses.

**Keyboard hook handles.** No leak. `UnhookWindowsHookEx` at `src-tauri/src/hotkey.rs:503`
runs on every cycle. The 1081 counter is a 9-hour uptime clock. See the premises section.

**Job object force-kill coverage.** Works, and this is verified on real evidence.
`src-tauri/src/jobobject.rs:19-34` creates one job with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`
and parks the handle in a `Lazy` static that is never closed, which is the correct pattern:
when the parent dies for any reason, including `TerminateProcess`, the OS closes the handle,
the job closes, and the children die with it. `assign_to_app_job` is called for each spawn at
`src-tauri/src/asr/whisper_server.rs:141-144`. Evidence it actually adopted the child:
`grep -c "AssignProcessToJobObject failed"` returns **0 across all four log files**, and that
warning is emitted on failure at `jobobject.rs:41`.

**`reqwest::Client` reuse.** Correct. One `Client` is built per `WhisperServer` in the
constructor (`src-tauri/src/asr/whisper_server.rs:71`) and reused for every request via
`self.http` (line 300). The per-request `.timeout(timeout)` at line 302 overrides the
duration without building a new client. Connection pooling therefore works, and no
per-request client or socket is leaked. `OpenAiCompat` follows the same pattern
(`src-tauri/src/asr/openai_compat.rs:20`).

**Temp WAV files.** There are none. `paths::temp_dir()` is created by `ensure_all`
(`src-tauri/src/paths.rs:57`, `72`) and **nothing ever writes to it**; the only references in
the whole tree are its own definition and that `create_dir_all`. The single recovery file is
`recovery/last.wav`, written at `src-tauri/src/pipeline.rs:573-575` and deleted on success at
line 813. Each dictation overwrites it. Directory confirmed empty on this
machine.

**Diagnostics event list.** Bounded and replaced, never appended.
`src/pages/Diagnostics.tsx:45` calls `api.debugEvents(300)` and `setEvents(parse(l))`
replaces the array wholesale. The backend caps it anyway at
`src-tauri/src/commands.rs:598` (`limit.unwrap_or(200).min(2000)`). There is no
`setInterval` on that page, and the five-click counter at line 57 self-trims to a 3-second
window. `list_history` is capped at 200 by default (`src-tauri/src/commands.rs:176`).

**Frontend listener cleanup.** All Tauri subscriptions are unlistened.
`src/App.tsx:47-52`, `src/pages/Home.tsx:34`, `src/pages/History.tsx:21`,
`src/overlay/main.tsx:111-114`. The overlay's level history is a fixed 28-entry ring
(`src/overlay/main.tsx:76-82`, push then shift).

**Thread census.** Four spawn sites, three of them bounded for the process lifetime:

| Thread | Where | Lifetime |
|---|---|---|
| `lalia-audio` | `src-tauri/src/audio.rs:107` | One, permanent |
| `lalia-keyboard-hook` | `src-tauri/src/hotkey.rs:526` | One, permanent |
| `lalia-clipboard` | `src-tauri/src/insertion.rs:425` | One, guarded by `OnceCell` at line 409 |
| UIA inspection | `src-tauri/src/context.rs:163` | **One per dictation.** See H-4 |

Plus two Tokio tasks per engine start draining the child's stdout and stderr
(`src-tauri/src/asr/whisper_server.rs:149`, `167`); both end when the pipes close, so they do
not survive a restart.

**The resampler's internal buffers.** Bounded. `Resampler::history` keeps at most
`taps.len() - 1` = 62 samples (`src-tauri/src/audio.rs:421-422`) and `pending` keeps the one
or two samples left after decimation (line 433-434). Neither grows across callbacks.

**The pre-roll ring.** Bounded and correctly enforced. `src-tauri/src/audio.rs:62-68` pops
from the front at capacity, and `configure` trims on shrink (lines 119-121). At the default
350 ms that is 5,600 samples, about 22 KB.

**Cleanup engines and regexes.** All `Lazy<Regex>` statics compiled once
(`src-tauri/src/cleanup/deterministic.rs:36-49`, `154-167`, and similar). `DictionaryEngine`
and `SnippetEngine` are rebuilt wholesale and the old value dropped in
`src-tauri/src/app.rs:38-44`. No growing cache anywhere in `cleanup/`.

---

# One thing to leave alone

**Do not switch SQLite to WAL.** `src-tauri/src/db.rs:283-286` sets
`journal_mode=DELETE` and `synchronous=FULL`, and the comment records why: on 2026-09-06 the
app came up after a reboot seeing an empty database while every row was still in the WAL.
The current setting costs an fsync per insert, which at 57 inserts/day is irrelevant. The
data-loss it prevents is not. This audit recommends no change here.

---

# Fix order

1. **H-1**, idle recycle for the engine child. This is the 1843 MB.
2. **L-1**, drop `lalia_lib=debug` in release. One line, and it cuts the log volume measured in H-3 by a large factor while taking work off the low-level hook path.
3. **H-4**, `CoUninitialize`, or better, one long-lived UIA thread.
4. **H-2**, move retention and backup pruning onto a periodic tick, and reconsider `retention_days: None` as a shipped default.
5. **M-1**, `stop()` on warm-up failure and a fresh port for the CPU fallback.
6. **M-3** and **M-4** together, pre-reserve the recording buffer and make `discard` consistent with `stop_recording`.
7. **M-2** and **M-5**, clear the retained buffers on every terminal path.

**H-3** needs nothing further; it was fixed while this was being written.
