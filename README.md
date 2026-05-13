# mdBook Relative Date Preprocessor

A simple Rust mdBook preprocessor that renders relative-date placeholders inside Markdown pages at build time.

## Behavior

Supported placeholders:

- `{{relative_to:2026_05_01}}`
- `{{ relative_to: 2026_05_01 }}`
- `{{relative_to:current}}`
- `{{ relative_to: current }}`

Rules:

- `YYYY_MM_DD` values are parsed as fixed dates.
- `current` means the latest git commit date touching the current page.
- If git commit info is unavailable, `current` falls back to the system date.
- Output is parenthesized, for example `(12 days ago)` or `(in 3 days)`.
- Relative text uses coarse units: days, weeks, months, years.
- Invalid tokens emit a build warning and are removed from output.

## Example

Markdown source:

```md
Published: {{relative_to:2026_05_01}}
Updated: {{relative_to:current}}
```

Possible rendered output:

```md
Published: (12 days ago)
Updated: (3 weeks ago)
```

## Usage

Build the preprocessor:

```sh
cargo build --release
```

Configure your `book.toml`:

```toml
[preprocessor.relative-date]
command = "/absolute/path/to/target/release/mdbook-relative-date"
```

For local development in this repository, `book.toml` uses:

```toml
[preprocessor.relative-date]
command = "cargo run --quiet --manifest-path ./Cargo.toml --"
```

Then build the demo book:

```sh
mdbook build
```

## Demo book

This repository includes a minimal mdBook demo:

- `book.toml`
- `book-src/SUMMARY.md`
- `book-src/demo.md`

Running `mdbook build` writes HTML output to `book/`.

## Git commit date lookup

For `{{relative_to:current}}`, the preprocessor runs:

```sh
git -C <book-root> log -1 --format=%cI -- <book-src>/<page-path>
```

The resulting commit timestamp is converted to a UTC date before calculating the relative date.

## Compatibility

This scaffold targets mdBook `0.5.x` via the `mdbook-preprocessor` crate.
