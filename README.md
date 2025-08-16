# f3rs_read

A fast Rust tool to **verify files/paths** created by F3-like write tools and detect data issues.  
It scans in sectors and reports how many sectors are **ok**, **changed**, **overwritten**, or **corrupted**, plus speed and ETA.

> This workspace also contains `f3write` (optional). Binaries are installed as **`f3rs_read`** (and optionally **`f3rs_write`**) via `[[bin]].name` in each package.

---

## Install

From the workspace root:

```bash
# Installing f3rs_write and f3rs_read
make install
```

Verify:
```bash
which f3rs_read
f3rs_read --help
```

---

## Usage

```
f3rs_read [OPTIONS] <PATH>...
```

Examples:
```bash
# Validate all .h2w files in current directory
f3rs_read .

# Validate a specific file
f3rs_read ./testfile.h2w

# Validate a mounted path/device
f3rs_read /mnt/usb1
```

Common options (if enabled in your build):
- `--sector-size <BYTES>` (default: 512)
- `--tolerance <N>` per-sector word mismatches allowed (default: 2)
- `--rate <MiB/s>` soft I/O rate limit
- `--no-progress` disable live progress
- `-v` / `-q` verbosity

Run `f3rs_read --help` for your exact list.

---

## Output (at a glance)

```
SECTORS      ok/corrupted/changed/overwritten
Validating file: X.h2w ...  2097152/        0/       0/       0
...
Data OK:: 10.05 GB
Data LOST:: 0 Bytes
Corrupted:: 0 Bytes
Slightly changed:: 0 Bytes
Overwritten:: 0 Bytes
Average speed: 49.42 MB/s
```

---

## Testing

Unit tests (all workspace members):
```bash
cargo test
```

Show test output live and run single-threaded:
```bash
cargo test -p f3core -- --nocapture --test-threads=1
```

---

## Run from workspace (dev)

```bash
# Writer
cargo run -p f3write --bin f3rs_write -- --help

# Reader
cargo run -p f3read --bin f3rs_read -- --help
cargo run -p f3read --bin f3rs_read -- .

```

---

## Logging

Use standard `log` macros; initialize one logger once at startup. Example (env_logger):

```bash
RUST_LOG=debug f3rs_read .
```

---

## Project layout

```
f3rs/
  Cargo.toml          # [workspace]
  f3core/             # library crate with shared logic
  f3read/             # reader binary (installs as `f3rs_read`)
  f3write/            # writer binary (optional, installs as `f3rs_write`)
```
