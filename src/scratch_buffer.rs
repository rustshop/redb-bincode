use std::cell::{RefCell, RefMut};
use std::ops::{Deref, DerefMut};
use std::thread::LocalKey;

thread_local! {
    static ENCODE_KEY: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static ENCODE_VALUE: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

struct ScratchBufferGuard<'a> {
    buffer: RefMut<'a, Vec<u8>>,
}

impl<'a> ScratchBufferGuard<'a> {
    fn new(mut buffer: RefMut<'a, Vec<u8>>) -> Self {
        buffer.clear();
        Self { buffer }
    }
}

impl Deref for ScratchBufferGuard<'_> {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl DerefMut for ScratchBufferGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

impl Drop for ScratchBufferGuard<'_> {
    fn drop(&mut self) {
        self.buffer.clear();
    }
}

fn with_scratch_buffer<R>(
    local: &'static LocalKey<RefCell<Vec<u8>>>,
    role: &'static str,
    f: impl FnOnce(&mut Vec<u8>) -> R,
) -> R {
    local.with(|buffer| {
        let buffer = buffer
            .try_borrow_mut()
            .unwrap_or_else(|_| panic!("reentrant {role} scratch-buffer access"));
        let mut buffer = ScratchBufferGuard::new(buffer);
        f(&mut buffer)
    })
}

pub(crate) fn with_encode_key_buf<R>(f: impl FnOnce(&mut Vec<u8>) -> R) -> R {
    with_scratch_buffer(&ENCODE_KEY, "key encoder", f)
}

pub(crate) fn with_encode_value_buf<R>(f: impl FnOnce(&mut Vec<u8>) -> R) -> R {
    with_scratch_buffer(&ENCODE_VALUE, "value encoder", f)
}
