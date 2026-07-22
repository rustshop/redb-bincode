use redb_bincode::{Database, TableDefinition, BINCODE_CONFIG};

const TEST_TABLE: TableDefinition<u64, u64> = TableDefinition::new("strict_decode");

#[test]
fn rejects_trailing_key_and_value_bytes() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let db = Database::create(dir.path().join("strict.redb"))?;

    let write = db.begin_write()?;
    {
        let mut table = write.open_table(&TEST_TABLE)?;
        table.insert(&7, &70)?;
    }
    write.commit()?;

    let canonical_key = bincode::encode_to_vec(7u64, BINCODE_CONFIG)?;
    let canonical_value = bincode::encode_to_vec(70u64, BINCODE_CONFIG)?;
    let second_key = bincode::encode_to_vec(8u64, BINCODE_CONFIG)?;
    let mut trailing_key = canonical_key.clone();
    trailing_key.push(0);
    let mut trailing_value = canonical_value.clone();
    trailing_value.push(0);

    let write = db.as_raw().begin_write()?;
    {
        let mut raw = write.open_table(TEST_TABLE.as_raw())?;
        raw.insert(trailing_key.as_slice(), canonical_value.as_slice())?;
        raw.insert(second_key.as_slice(), trailing_value.as_slice())?;
    }
    write.commit()?;

    let read = db.begin_read()?;
    let table = read.open_table(&TEST_TABLE)?;

    let canonical_range = table.range(7..=7)?.collect::<Result<Vec<_>, _>>()?;
    assert_eq!(canonical_range.len(), 1);
    assert_eq!(canonical_range[0].0.value_try()?, 7);
    assert_eq!(canonical_range[0].1.value_try()?, 70);

    let all = table.range::<u64>(..)?.collect::<Result<Vec<_>, _>>()?;
    assert_eq!(all.len(), 3);
    assert!(all.iter().any(|(key, _)| key.value_try().is_err()));
    assert!(table
        .get(&8)?
        .expect("raw row must exist")
        .value_try()
        .is_err());

    Ok(())
}
