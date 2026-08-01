# Repository guidance

## Required reading

Read [SECURITY.md](SECURITY.md) before modifying this repository.


## Persistent-data contracts

Preserve complete-slice decoding at every typed decode site. Stored keys and
values must contain exactly one canonical `BINCODE_CONFIG` item; never accept
trailing bytes.

Validate keys inside checked `AccessGuard` constructors before returning them
from `first()`, `last()`, or either range direction. Malformed keys must panic
before an entry becomes observable, including value-only iteration. Keep value
decoding lazy so callers can handle malformed values through `value_try()`.


## Scratch-buffer contracts

Keep encoder scratch buffers private. Key-to-value nesting is supported, but
same-role reentrancy must fail before creating aliased mutable access. Clear
contents after success, returned errors, and unwinding while retaining
capacity. Tests must inspect post-operation state without reacquiring a guard,
because acquisition itself clears the buffer.


## Verification

Run `cargo fmt --all -- --check`, Clippy with warnings denied, all-target tests,
`cargo package`, and `selfci check` for release changes.
