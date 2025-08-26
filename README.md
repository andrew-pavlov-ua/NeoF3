# NeoF3 — Fast Flash Fake Finder (Rust)

**NeoF3** is a Rust workspace that reimagines the classic F3 utilities:

- **`f3rs_write`** — fills free space with numbered test files (e.g., `12.h2w`) to benchmark throughput and prepare data for verification.
- **`nf3_read`** — verifies those files/paths, detecting **ok**, **changed**, **overwritten**, and **corrupted** sectors, and reports speed/ETA.

> ⚠️ **Safety note:** The writer can fill your target filesystem completely. Double‑check the path/mount you test and keep other apps closed to avoid running out of space during a run.

---

## Features

### f3rs_write (writer)

- Sequentially writes numbered files (`<index>.h2w`) until free space is exhausted or an explicit end index is reached.
- Live progress: instantaneous & average speed, ETA, elapsed time.
- Resume‑friendly: start/stop by file index.
- Optional rate limiting (soft) for reproducible benchmarking.

### nf3_read (reader)

- Scans files/paths in fixed‑size sectors and classifies each sector as:
  - **ok** — matches the expected pattern,
  - **corrupted** — random/invalid content,
  - **changed** — tag matches, payload altered,
  - **overwritten** — valid pattern of a *different* sector (aliasing/wrap‑around).
- Aggregated per‑file and total stats; speed and ETA reporting.
- Works on single files or whole paths (e.g., a mountpoint).

---

## Install

From the workspace root:

```bash
# Installs both binaries if you have a helper like `just`:
just install
```

Or build manually:

```bash
cargo build --release
# Binaries will be in target/release/
```

Verify:

```bash
which f3rs_write
f3rs_write --help

which nf3_read
nf3_read --help
```

> The binaries are named **`f3rs_write`** and **`nf3_read`** via each package’s `[[bin]].name`.

---

## Usage

### Writer — f3rs_write

```bash
f3rs_write [OPTIONS] [PATH]
```

Typical options (exact list depends on your build; run `--help`):

- `-s, --start-at <NUM>` — first `<NUM>.h2w` to write (default: `1`)
- `-e, --end-at <NUM>` — last `<NUM>.h2w` to write (default: `0` = auto/ignore)
- `--show-progress=<BOOL>` — enable/disable live progress (default: `true`)
- `-w, --max-write-rate <KBPS>` — soft limit write rate in KB/s (default: `0` = unlimited)
- `PATH` — directory/mount to write files into (default: current dir)

Examples:

```bash
# Fill current directory with numbered .h2w files
f3rs_write

# Write a single file 12.h2w to a USB drive
f3rs_write -s 12 -e 12 /media/USB
```

**Sample output:**

```bash
Version: 0.1.0
Free space: 14.46875 MB
start_at: 12
end_at: 12
Filling file: 12.h2w                                                                                                      - OK
Free space available: 0 Bytes
Average speed: 18.56 MB/s
Total elapsed: 462.08ms
```

---

### Reader — nf3_read

```bash
nf3_read [OPTIONS] <PATH>...
```

Typical options (run `--help` for your build):

- `-s, --start-at <NUM>` / `-e, --end-at <NUM>` — index range to verify
- `--show-progress=<BOOL>` — enable/disable live progress (default: `true`)
- `-r, --max-read-rate <KBPS>` — soft limit read rate in KB/s (default: `0` = unlimited)
- `PATH...` — one or multiple files/paths to verify

Examples:

```bash
# Verify all .h2w files in current directory
nf3_read .

# Verify a single file
nf3_read ./12.h2w

# Verify an entire mounted path
nf3_read /mnt/usb1
```

**Output at a glance:**

```bash
        SECTORS      ok/corrupted/changed/overwritten
Validating file: 12.h2w ...        2097152/       0/      0/       0
...
Data OK:            10.05 GB
Data LOST:          0 Bytes
Corrupted:          0 Bytes
Slightly changed:   0 Bytes
Overwritten:        0 Bytes
Average speed:      49.42 MB/s
```

---

## Tips for accurate results & performance

- **Writer (`f3rs_write`)**
  - Prefer a larger block size (1–8 MiB) internally; avoid per‑block `fsync` — sync at file end.
  - If you need to force durability, use `sync_data()` on the file and optionally fsync the parent directory.
  - Keep progress rendering out of the hot write path (separate thread).

- **Reader (`nf3_read`)**
  - If you just wrote the files and want *cold* reads, consider dropping the file’s page cache (Unix `posix_fadvise(..., DONTNEED)`) once before starting, or periodically in big chunks.
  - Do sequential reads with a large reusable buffer (1–8 MiB). `BufReader` helps only for tiny reads; otherwise read directly.
  - For raw‑device‑like speeds on Linux, `O_DIRECT` is possible but requires 4KiB alignment (advanced).

- **General**
  - Run in **release** mode for realistic performance: `cargo run --release -p f3write`.
  - Close all writer handles before verification.
  - Ensure target path is correct to avoid filling your system drive.

---

## Project layout

```bash
NeoF3/
  Cargo.toml          # [workspace]
  f3core/             # shared library logic (no clap in core)
  f3write/            # writer binary (installs as `f3rs_write`)
  f3read/             # reader binary (installs as `nf3_read`)
```

You can keep CLI types (shared flags) in a gated module of `f3core` (feature `cli`) or a small helper crate (`f3cli`) and compose per‑tool parsers via `#[command(flatten)]`.

---

## Development & Testing

```bash
# lint & format
cargo fmt
cargo clippy --workspace --all-targets --all-features -- -D warnings

# tests
cargo test --workspace
# show test output live / single-threaded for deterministic logs
cargo test -p f3core -- --nocapture --test-threads=1
```

**Run from workspace (dev):**

```bash
# Writer
cargo run -p f3write --bin f3rs_write -- --help

# Reader
cargo run -p f3read  --bin nf3_read  -- --help
cargo run -p f3read  --bin nf3_read  -- .
```

---

## Logging

Use standard `log` macros and initialize one logger once (e.g., `env_logger`):

```bash
RUST_LOG=debug nf3_read .
```

---

## License & contributions

Choose and include a license (`MIT`/`Apache-2.0`/`GPL-3.0` etc.). PRs and issues are welcome. Please keep changes focused, add tests where possible, and run `fmt`/`clippy` before submitting.
