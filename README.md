# mdBook Relative Date Preprocessor

A simple mdBook preprocessor plugin (Rust) to render relative-date placeholders inside Markdown pages.

## Agreed Behavior

### 1) Preprocessor type
- Implemented as a **Rust mdBook preprocessor**.
- Runs at **build time** only.

### 2) Supported placeholder syntax
Both compact and spaced forms are supported:
- `{{relative_to:2026_05_01}}`
- `{{ relative_to: 2026_05_01 }}`
- `{{relative_to:current}}`
- `{{ relative_to: current }}`

### 3) Placeholder semantics
#### `{{relative_to:YYYY_MM_DD}}`
- Parses date in `YYYY_MM_DD` format.
- Renders as parenthesized relative time, e.g. `(12 days ago)`.

#### `{{relative_to:current}}`
- Means: **time since the current page’s latest git commit**.
- For each page, use the latest commit date that touched that page file.

### 4) Date source and fallback
- Primary source for `current`: latest commit date for that page.
- If git date is unavailable, fallback to **system current date**.

### 5) Render format
- Output style: **parenthesized only**.
  - Example: `(2 weeks ago)`
  - Example future date: `(in 3 days)`

### 6) Granularity
Use coarse humanized units:
- days
- weeks
- months / years

### 7) Error/edge-case behavior
- Invalid date token (not `YYYY_MM_DD`):
  - Emit a build warning.
  - Remove the token text from rendered output.
- Future dates:
  - Render with natural future wording, e.g. `(in 3 days)`.

## Example
Input markdown:

```md
Published: {{relative_to:2026_05_01}}
Updated: {{relative_to:current}}
```

Possible rendered output:

```md
Published: (12 days ago)
Updated: (3 weeks ago)
```
