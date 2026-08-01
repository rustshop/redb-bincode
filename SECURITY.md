# Security policy

## Reporting vulnerabilities

Report suspected vulnerabilities privately through a
[GitHub security advisory](https://github.com/rustshop/redb-bincode/security/advisories/new).
Do not include sensitive vulnerability details in a public issue.


## Persistent-data trust boundary

`redb-bincode` assumes that typed tables contain canonical data written with
the crate's current big-endian `BINCODE_CONFIG`. Typed decoding requires each
stored slice to contain exactly one complete item; trailing bytes are invalid.

Malformed values can be handled through `AccessGuard::value_try()`, which
returns `bincode::error::DecodeError`. Malformed keys panic when `first()`,
`last()`, or either direction of range iteration encounters them, before an
entry is yielded. Treat database files from untrusted sources as untrusted
input. Validate, migrate, or recover them with raw `redb` APIs before exposing
them through typed tables.

If persistent data is corrupted, stop typed access, preserve the original file
for diagnosis, and restore a known-good backup or copy validated entries into a
new database through raw access. Catching a malformed-key panic does not repair
the stored data.


## Panic and scratch-buffer recovery

Typed encoding reuses private thread-local key and value scratch buffers.
Separate roles may nest, but recursively acquiring the same role panics before
mutable access can alias. Guards clear buffer contents on success, returned
errors, and unwinding. Capacity remains cached per thread. With
`panic = "abort"`, destructors do not run because the process terminates.
