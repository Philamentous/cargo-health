> **Disclaimer:** If you didn't see my profile description. I am a biologist. I have some computer science background, but not coding. I am publishing some cargo crates and other little repos to (hopefully) meaningfully contribute to open-source projects (tactfully, I hope) and rust in general with any extra claude credits I have available. I am trying to ensure that any contributions I make are actually helpful so any criticism or feedback of my approach would be greatly appreciated.

# cargo-health

A cargo subcommand that scans your dependency tree and reports maintenance health.

## What it does

`cargo-health` parses your `Cargo.lock` file, queries the crates.io API for each dependency, and scores them based on maintenance signals:

- **Update recency** (50 points): How recently the crate was updated. Full score within 90 days, linearly decaying to zero at 2 years.
- **Download count** (30 points): Logarithmic scale based on total downloads. 30 points at 1M+ downloads.
- **Repository presence** (20 points): Whether the crate has a linked repository URL.

Dependencies are categorized as:
- **HEALTHY** (score >= 60): Well-maintained and widely used
- **WARNING** (score 30-59): May need attention
- **CRITICAL** (score < 30): Potentially abandoned or unmaintained

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Run in any project directory with a Cargo.lock
cargo health

# Only show warnings and critical issues
cargo health --warn-only

# Specify a custom lockfile path
cargo health --lockfile path/to/Cargo.lock
```

## Example Output

```
=== Dependency Health Report ===

  [HEALTHY] serde v1.0.193 (score: 95, updated: 12 days ago, downloads: 150.2M, repo)
  [HEALTHY] clap v4.4.11 (score: 88, updated: 30 days ago, downloads: 45.3M, repo)
  [WARNING] old-crate v0.3.1 (score: 42, updated: 400 days ago, downloads: 5.2K, repo)
  [CRITICAL] abandoned v0.1.0 (score: 8, updated: 900 days ago, downloads: 120, no repo)

--- Summary ---
  2 healthy | 1 warning | 1 critical
  4 total dependencies scanned
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
