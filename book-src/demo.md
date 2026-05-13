# Relative Date Demo

This page demonstrates `mdbook-relative-date` placeholders.

## Explicit date

A fixed date renders here:

{{relative_to:2026_05_01}}

The spaced placeholder form also renders here:

{{ relative_to: 2026_05_01 }}

## Future date

A future date renders with natural wording:

{{relative_to:2999_01_01}}

## Current page commit date

The `current` keyword uses the latest git commit date touching this page.
If git information is unavailable, it falls back to the system date:

{{relative_to:current}}

## Invalid token

The following invalid token should disappear and emit a build warning:

{{relative_to:not_a_date}}
