use std::path::Path;

use redb::{DatabaseError, TransactionError};

use super::tx::{ReadTransaction, WriteTransaction};
use crate::tx;

#[derive(Debug)]
pub struct Database(redb::Database);

impl Database {
    pub fn create(path: impl AsRef<Path>) -> Result<Database, DatabaseError> {
        Ok(Self(redb::Database::create(path)?))
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Database, DatabaseError> {
        Ok(Self(redb::Database::open(path)?))
    }

    pub fn begin_read(&self) -> Result<tx::ReadTransaction, TransactionError> {
        Ok(ReadTransaction::from(self.0.begin_read()?))
    }

    pub fn begin_write(&self) -> Result<tx::WriteTransaction, TransactionError> {
        Ok(WriteTransaction::from(self.0.begin_write()?))
    }

    /// Get the inner [`redb::Database`]
    pub fn as_raw(&self) -> &redb::Database {
        &self.0
    }
    pub fn as_raw_mut(&mut self) -> &mut redb::Database {
        &mut self.0
    }
}

impl From<redb::Database> for Database {
    fn from(value: redb::Database) -> Self {
        Self(value)
    }
}
