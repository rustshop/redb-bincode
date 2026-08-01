use std::panic::{catch_unwind, AssertUnwindSafe};

use redb_bincode::{Database, TableDefinition, BINCODE_CONFIG};

const CANONICAL: TableDefinition<u64, u64> = TableDefinition::new("canonical");
const MALFORMED_KEY: TableDefinition<u64, u64> = TableDefinition::new("malformed_key");
const MALFORMED_VALUE: TableDefinition<u64, u64> = TableDefinition::new("malformed_value");
const RETAIN_KEY: TableDefinition<u64, u64> = TableDefinition::new("retain_key");
const RETAIN_VALUE: TableDefinition<u64, u64> = TableDefinition::new("retain_value");
const RETAIN_IN_KEY: TableDefinition<u64, u64> = TableDefinition::new("retain_in_key");
const RETAIN_IN_VALUE: TableDefinition<u64, u64> = TableDefinition::new("retain_in_value");

fn assert_panics(f: impl FnOnce()) {
    assert!(catch_unwind(AssertUnwindSafe(f)).is_err());
}

fn create_database() -> anyhow::Result<(tempfile::TempDir, Database)> {
    let dir = tempfile::tempdir()?;
    let db = Database::create(dir.path().join("strict.redb"))?;
    let key = bincode::encode_to_vec(7u64, BINCODE_CONFIG)?;
    let value = bincode::encode_to_vec(70u64, BINCODE_CONFIG)?;
    let mut trailing_key = key.clone();
    trailing_key.push(0);
    let mut trailing_value = value.clone();
    trailing_value.push(0);

    let write = db.as_raw().begin_write()?;
    {
        write
            .open_table(CANONICAL.as_raw())?
            .insert(key.as_slice(), value.as_slice())?;
        write
            .open_table(MALFORMED_KEY.as_raw())?
            .insert(trailing_key.as_slice(), value.as_slice())?;
        write
            .open_table(MALFORMED_VALUE.as_raw())?
            .insert(key.as_slice(), trailing_value.as_slice())?;
        write
            .open_table(RETAIN_KEY.as_raw())?
            .insert(trailing_key.as_slice(), value.as_slice())?;
        write
            .open_table(RETAIN_VALUE.as_raw())?
            .insert(key.as_slice(), trailing_value.as_slice())?;
        write
            .open_table(RETAIN_IN_KEY.as_raw())?
            .insert(trailing_key.as_slice(), value.as_slice())?;
        write
            .open_table(RETAIN_IN_VALUE.as_raw())?
            .insert(key.as_slice(), trailing_value.as_slice())?;
    }
    write.commit()?;
    Ok((dir, db))
}

#[test]
fn complete_slice_decoding_rejects_trailing_key_and_value_bytes() -> anyhow::Result<()> {
    let (_dir, db) = create_database()?;
    let read = db.begin_read()?;

    let canonical = read.open_table(&CANONICAL)?;
    let entries = canonical.range(7..=7)?.collect::<Result<Vec<_>, _>>()?;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0.value_try()?, 7);
    assert_eq!(entries[0].1.value_try()?, 70);

    let malformed_value = read.open_table(&MALFORMED_VALUE)?;
    assert!(malformed_value
        .get(&7)?
        .expect("raw row exists")
        .value_try()
        .is_err());

    Ok(())
}

#[test]
fn malformed_keys_panic_at_every_table_key_boundary() -> anyhow::Result<()> {
    let (_dir, db) = create_database()?;

    let write = db.begin_write()?;
    {
        let table = write.open_table(&MALFORMED_KEY)?;
        assert_panics(|| {
            let _ = table.first();
        });
        assert_panics(|| {
            let _ = table.last();
        });
        assert_panics(|| {
            let _ = table
                .range::<u64>(..)
                .expect("range opens")
                .next()
                .map(|entry| entry.expect("storage remains readable").1.value());
        });
        assert_panics(|| {
            let _ = table
                .range::<u64>(..)
                .expect("range opens")
                .next_back()
                .map(|entry| entry.expect("storage remains readable").1.value());
        });
    }
    drop(write);

    let read = db.begin_read()?;
    let table = read.open_table(&MALFORMED_KEY)?;
    assert_panics(|| {
        let _ = table.first();
    });
    assert_panics(|| {
        let _ = table.last();
    });
    assert_panics(|| {
        let _ = table
            .range::<u64>(..)
            .expect("range opens")
            .next()
            .map(|entry| entry.expect("storage remains readable").1.value());
    });
    assert_panics(|| {
        let _ = table
            .range::<u64>(..)
            .expect("range opens")
            .next_back()
            .map(|entry| entry.expect("storage remains readable").1.value());
    });

    Ok(())
}

#[test]
fn retain_operations_reject_trailing_bytes() -> anyhow::Result<()> {
    let (_dir, db) = create_database()?;

    let write = db.begin_write()?;
    {
        let mut table = write.open_table(&RETAIN_KEY)?;
        assert_panics(|| {
            let _ = table.retain(|_, _| true);
        });
    }
    drop(write);

    let write = db.begin_write()?;
    {
        let mut table = write.open_table(&RETAIN_VALUE)?;
        assert_panics(|| {
            let _ = table.retain(|_, _| true);
        });
    }
    drop(write);

    let write = db.begin_write()?;
    {
        let mut table = write.open_table(&RETAIN_IN_KEY)?;
        assert_panics(|| {
            let _ = table.retain_in(.., |_: &u64, _: &u64| true);
        });
    }
    drop(write);

    let write = db.begin_write()?;
    {
        let mut table = write.open_table(&RETAIN_IN_VALUE)?;
        assert_panics(|| {
            let _ = table.retain_in(.., |_: &u64, _: &u64| true);
        });
    }

    Ok(())
}
