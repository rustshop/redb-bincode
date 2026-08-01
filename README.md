# `redb-bincode`

This crate is a wrapper around [`redb`](https://crates.io/crates/redb)
that makes keys and values require `T: bincode::Encode + bincode::Decode`
and serialize as a big-endian `bincode`, which makes working with `redb`
much more convenient.

It does also include a couple of minor deviations from `redb`, but stays
faithful to the original API.

## Encoder scratch-buffer behavior

Typed table operations reuse separate key and value buffers on each thread.
Key-to-value nesting used by inserts and bounded ranges is supported. Recursive
use of the same buffer role panics before creating aliased mutable access.

The buffers are cleared by an unwind guard after successful operations, storage
errors, and codec panics. With `panic = "abort"`, no recovery runs because the
process terminates. Each thread retains its peak key and value buffer capacity
for reuse.

It was born from personal need, and at the time of writing this,
contains only things that I actually needed. Having said that adding
missing bits should be as simple as copy-paste-modify existing code,
and contributions are welcome. Just be aware that you might need to
get your hands dirty if you're planning to use it.


## AI usage disclosure

[I use LLMs when working on my projects.](https://dpc.pw/posts/personal-ai-usage-disclosure/)
