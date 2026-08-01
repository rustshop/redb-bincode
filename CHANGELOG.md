# Changelog

## 0.5.0

* **Breaking:** Removed the public `ENCODE_KEY` and `ENCODE_VALUE` thread-local
  scratch buffers. Typed operations now use private, checked scratch buffers
  that reject same-role reentrancy and reliably reset after errors and unwinding.
* Rejected stored keys and values whose bincode representation contains trailing
  bytes. Decoding now requires the complete stored slice to contain exactly one
  encoded item.
* Validated keys eagerly in `first()`, `last()`, and both directions of range
  iteration. Malformed keys now panic before yielding an entry, even for
  value-only consumption. Values remain lazily decoded: `value_try()` reports
  malformed values as `bincode::error::DecodeError`.
