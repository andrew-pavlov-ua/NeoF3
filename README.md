# NeoF3 --- Fast Flash Fake Finder (Rust)

**NeoF3** is a Rust implementation of the classic F3 utilities for
detecting counterfeit and unreliable flash storage.

The recommended interface is a single executable with subcommands:

``` bash
nf3 write
nf3 read
```

For backward compatibility, the standalone binaries are still supported:

``` bash
nf3_write
nf3_read
```

Currently available tools:

-   **`nf3 write`** --- fills free space with numbered test files
    (`*.h2w`) to prepare a filesystem for verification.
-   **`nf3 read`** --- verifies those files and detects corruption,
    overwritten sectors, and counterfeit flash behavior.

> **Why a unified CLI?**
>
> Starting with v0.2, NeoF3 provides a single `nf3` executable with
> subcommands (`write`, `read`, and future tools such as `probe`, `fix`,
> and `brew`). The legacy binaries remain available for backwards
> compatibility.

> ⚠️ **Safety note:** The writer can fill the target filesystem
> completely. Always verify the destination path before starting a test.

## Features

### `nf3 write`

-   Sequentially writes numbered `.h2w` files.
-   Resume-friendly via start/end file indexes.
-   Live progress with speed, ETA and elapsed time.
-   Optional write-rate limiting.

### `nf3 read`

-   Verifies files or entire directories.
-   Detects:
    -   **ok**
    -   **corrupted**
    -   **changed**
    -   **overwritten**
-   Live progress with throughput and ETA.
-   Aggregated per-file and total statistics.

## Install

```bash
# Installs both binaries if you have a helper like `just`:
just install
```

Or build manually:

```bash
cargo build --release
```

Generated executables:

``` text
target/release/nf3
target/release/nf3_write
target/release/nf3_read
```

Verify installation:

``` bash
nf3 --help
nf3 write --help
nf3 read --help

nf3_write --help
nf3_read --help
```

## Usage

### Preferred CLI

#### Write

``` bash
nf3 write [OPTIONS] [PATH]
```

Examples:

``` bash
nf3 write
nf3 write /media/USB
nf3 write -s 12 -e 12 /media/USB
```

#### Read

``` bash
nf3 read [OPTIONS] <PATH>...
```

Examples:

``` bash
nf3 read .
nf3 read ./12.h2w
nf3 read /mnt/usb1
```

## Legacy CLI

The previous command names continue to work:

``` bash
nf3_write [OPTIONS] [PATH]
nf3_read [OPTIONS] <PATH>...
```

  New                      Legacy
  ------------------------ ------------------------
  `nf3 write /media/USB`   `nf3_write /media/USB`
  `nf3 write -s 12`        `nf3_write -s 12`
  `nf3 read .`             `nf3_read .`
  `nf3 read /mnt/usb`      `nf3_read /mnt/usb`

## Project layout

``` text
NeoF3/
├── Cargo.toml
├── nf3/
├── f3core/
├── f3write/
├── f3read/
└── ...
```

## Development

``` bash
cargo fmt
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Run the unified CLI:

``` bash
cargo run -p nf3 -- write --help
cargo run -p nf3 -- read --help
cargo run -p nf3 -- read .
```

Legacy wrappers:

``` bash
cargo run -p f3write
cargo run -p f3read
```

## Logging

``` bash
RUST_LOG=debug nf3 read .
```

## License

Choose and include a license (`MIT`/`Apache-2.0`/`GPL-3.0` etc.). PRs and issues are welcome. Please keep changes focused, add tests where possible, and run `fmt`/`clippy` before submitting.
