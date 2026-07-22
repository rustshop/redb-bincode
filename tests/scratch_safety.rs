use std::any::Any;
use std::cell::RefCell;
use std::panic::{catch_unwind, AssertUnwindSafe};

use bincode::{Decode, Encode};
use redb_bincode::{Database, TableDefinition};

thread_local! {
    static REENTRY_DATABASE: RefCell<Option<(tempfile::TempDir, Database)>> =
        const { RefCell::new(None) };
}

#[derive(Debug, Decode, PartialEq, Eq)]
struct FragileEncode {
    value: u64,
    panic_after_prefix: bool,
}

impl Encode for FragileEncode {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.value.encode(encoder)?;
        self.panic_after_prefix.encode(encoder)?;
        assert!(!self.panic_after_prefix, "simulated encoder panic");
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct FragileDecode(u64);

impl Encode for FragileDecode {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.0.encode(encoder)
    }
}

impl<Context> Decode<Context> for FragileDecode {
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let value = u64::decode(decoder)?;
        assert_ne!(value, 99, "simulated decoder panic");
        Ok(Self(value))
    }
}

#[derive(Debug, Decode)]
struct ReentrantEncode(u64);

impl Encode for ReentrantEncode {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.0.encode(encoder)?;
        REENTRY_DATABASE.with(|database| {
            let database = database.borrow();
            let (_, database) = database.as_ref().expect("reentry database initialized");
            let read = database.begin_read().expect("reentry read transaction");
            let table = read
                .open_table(&REENTRY_TARGET_TABLE)
                .expect("reentry table");
            let _ = table.get(&1).expect("reentry point lookup");
        });
        Ok(())
    }
}

const KEY_PANIC_TABLE: TableDefinition<FragileEncode, u64> = TableDefinition::new("key_panic");
const VALUE_PANIC_TABLE: TableDefinition<u64, FragileEncode> = TableDefinition::new("value_panic");
const DECODE_PANIC_TABLE: TableDefinition<u64, FragileDecode> =
    TableDefinition::new("decode_panic");
const REENTRANT_TABLE: TableDefinition<ReentrantEncode, u64> = TableDefinition::new("reentrant");
const REENTRY_TARGET_TABLE: TableDefinition<u64, u64> = TableDefinition::new("reentry_target");

fn panic_message(payload: &(dyn Any + Send)) -> Option<&str> {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
}

#[test]
fn codec_panics_and_reentrancy_leave_scratch_buffers_reusable() -> anyhow::Result<()> {
    let reentry_dir = tempfile::tempdir()?;
    let reentry_db = Database::create(reentry_dir.path().join("reentry.redb"))?;
    let reentry_write = reentry_db.begin_write()?;
    {
        reentry_write
            .open_table(&REENTRY_TARGET_TABLE)?
            .insert(&1, &10)?;
    }
    reentry_write.commit()?;
    REENTRY_DATABASE.with(|database| {
        *database.borrow_mut() = Some((reentry_dir, reentry_db));
    });

    let dir = tempfile::tempdir()?;
    let db = Database::create(dir.path().join("scratch.redb"))?;
    let write = db.begin_write()?;

    {
        let mut table = write.open_table(&KEY_PANIC_TABLE)?;
        let panic = catch_unwind(AssertUnwindSafe(|| {
            let _ = table.insert(
                &FragileEncode {
                    value: 7,
                    panic_after_prefix: true,
                },
                &70,
            );
        }));
        assert!(panic.is_err());

        let good_key = FragileEncode {
            value: 7,
            panic_after_prefix: false,
        };
        table.insert(&good_key, &70)?;
        assert_eq!(table.get(&good_key)?.expect("inserted key").value(), 70);
    }

    {
        let mut table = write.open_table(&VALUE_PANIC_TABLE)?;
        let panic = catch_unwind(AssertUnwindSafe(|| {
            let _ = table.insert(
                &1,
                &FragileEncode {
                    value: 10,
                    panic_after_prefix: true,
                },
            );
        }));
        assert!(panic.is_err());

        let good_value = FragileEncode {
            value: 20,
            panic_after_prefix: false,
        };
        table.insert(&2, &good_value)?;
        assert_eq!(table.get(&2)?.expect("inserted value").value(), good_value);
    }

    {
        let mut table = write.open_table(&DECODE_PANIC_TABLE)?;
        table.insert(&1, &FragileDecode(99))?;
        table.insert(&2, &FragileDecode(20))?;
        let panic = catch_unwind(AssertUnwindSafe(|| {
            let _ = table.retain_in(1..=2, |_, _| true);
        }));
        assert!(panic.is_err());

        table.insert(&300, &FragileDecode(30))?;
        assert_eq!(
            table.get(&300)?.expect("scratch remains reusable").value(),
            FragileDecode(30)
        );
    }

    {
        let mut table = write.open_table(&REENTRANT_TABLE)?;
        let panic = catch_unwind(AssertUnwindSafe(|| {
            let _ = table.insert(&ReentrantEncode(1), &10);
        }))
        .expect_err("same-role recursive encoding must panic");
        assert_eq!(
            panic_message(&*panic),
            Some("reentrant key encoder scratch-buffer access")
        );
    }

    {
        let table = write.open_table(&KEY_PANIC_TABLE)?;
        let good_key = FragileEncode {
            value: 7,
            panic_after_prefix: false,
        };
        assert_eq!(
            table
                .get(&good_key)?
                .expect("scratch remains usable")
                .value(),
            70
        );
    }

    write.commit()?;
    REENTRY_DATABASE.with(|database| {
        database.borrow_mut().take();
    });
    Ok(())
}
