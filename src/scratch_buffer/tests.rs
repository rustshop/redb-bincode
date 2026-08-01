use std::panic::{catch_unwind, AssertUnwindSafe};

use super::{with_encode_key_buf, with_encode_value_buf, ENCODE_KEY, ENCODE_VALUE};

fn assert_buffers_empty() {
    ENCODE_KEY.with(|buffer| assert!(buffer.borrow().is_empty()));
    ENCODE_VALUE.with(|buffer| assert!(buffer.borrow().is_empty()));
}

#[test]
fn same_role_reentrancy_panics_without_aliasing_and_clears_outer_state() {
    let key_panic = catch_unwind(AssertUnwindSafe(|| {
        with_encode_key_buf(|outer| {
            outer.extend_from_slice(b"stale prefix");
            with_encode_key_buf(|_| unreachable!("same-role reentrancy must fail"));
        });
    }));
    assert!(key_panic.is_err());
    assert_buffers_empty();

    with_encode_key_buf(|buffer| {
        assert!(buffer.is_empty());
        buffer.extend_from_slice(b"usable");
    });
    assert_buffers_empty();

    let value_panic = catch_unwind(AssertUnwindSafe(|| {
        with_encode_value_buf(|outer| {
            outer.extend_from_slice(b"stale prefix");
            with_encode_value_buf(|_| unreachable!("same-role reentrancy must fail"));
        });
    }));
    assert!(value_panic.is_err());
    assert_buffers_empty();
}

#[test]
fn callback_panic_clears_key_and_value_state() {
    let key_panic = catch_unwind(AssertUnwindSafe(|| {
        with_encode_key_buf(|buffer| {
            buffer.extend_from_slice(b"stale prefix");
            panic!("simulated key codec panic");
        });
    }));
    assert!(key_panic.is_err());
    assert_buffers_empty();

    let value_panic = catch_unwind(AssertUnwindSafe(|| {
        with_encode_value_buf(|buffer| {
            buffer.extend_from_slice(b"stale prefix");
            panic!("simulated value codec panic");
        });
    }));
    assert!(value_panic.is_err());
    assert_buffers_empty();
}

#[test]
fn distinct_key_and_value_buffers_support_nested_access() {
    with_encode_key_buf(|key| {
        key.extend_from_slice(b"key");
        with_encode_value_buf(|value| {
            assert_eq!(key, b"key");
            assert!(value.is_empty());
            value.extend_from_slice(b"value");
        });
        assert_eq!(key, b"key");
    });

    assert_buffers_empty();
}

#[test]
fn success_and_error_results_clear_key_and_value_state() {
    with_encode_key_buf(|buffer| buffer.extend_from_slice(b"success"));
    with_encode_value_buf(|buffer| buffer.extend_from_slice(b"success"));
    assert_buffers_empty();

    let key_error = with_encode_key_buf(|buffer| {
        buffer.extend_from_slice(b"error");
        Err::<(), _>("simulated storage error")
    });
    assert_eq!(key_error, Err("simulated storage error"));

    let value_error = with_encode_value_buf(|buffer| {
        buffer.extend_from_slice(b"error");
        Err::<(), _>("simulated storage error")
    });
    assert_eq!(value_error, Err("simulated storage error"));
    assert_buffers_empty();
}
