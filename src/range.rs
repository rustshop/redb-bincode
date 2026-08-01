use std::fmt;
use std::marker::PhantomData;

use redb::StorageError;

use crate::{AccessGuard, SortKey, SortOrder};

pub struct Range<'a, K, V, IK = &'static [u8]>
where
    IK: redb::Value + 'static + redb::Key,
{
    inner: redb::Range<'a, IK, &'static [u8]>,
    _k: PhantomData<K>,
    _v: PhantomData<V>,
}

impl<'a, S, K, V> From<redb::Range<'a, SortKey<S>, &'static [u8]>> for Range<'a, K, V, SortKey<S>>
where
    S: SortOrder + fmt::Debug,
{
    fn from(inner: redb::Range<'a, SortKey<S>, &'static [u8]>) -> Self {
        Self {
            inner,
            _k: PhantomData,
            _v: PhantomData,
        }
    }
}

impl<'a, K, V> Iterator for Range<'a, K, V, &'static [u8]>
where
    K: bincode::Decode<()>,
{
    type Item = Result<(AccessGuard<'a, K>, AccessGuard<'a, V>), StorageError>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.inner.next()?.map(|(k, v)| {
            let key = AccessGuard::<K>::from_key(k);
            (key, AccessGuard::from(v))
        }))
    }
}

impl<'a, S, K, V> Iterator for Range<'a, K, V, SortKey<S>>
where
    S: SortOrder + fmt::Debug,
    K: bincode::Decode<()>,
{
    type Item = Result<(AccessGuard<'a, K, SortKey<S>>, AccessGuard<'a, V>), StorageError>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.inner.next()?.map(|(k, v)| {
            let key = AccessGuard::<K, SortKey<S>>::from_key(k);
            (key, AccessGuard::from(v))
        }))
    }
}
impl<K, V> DoubleEndedIterator for Range<'_, K, V, &'static [u8]>
where
    K: bincode::Decode<()>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(self.inner.next_back()?.map(|(k, v)| {
            let key = AccessGuard::<K>::from_key(k);
            (key, AccessGuard::from(v))
        }))
    }
}

impl<S, K, V> DoubleEndedIterator for Range<'_, K, V, SortKey<S>>
where
    S: SortOrder + fmt::Debug,
    K: bincode::Decode<()>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(self.inner.next_back()?.map(|(k, v)| {
            let key = AccessGuard::<K, SortKey<S>>::from_key(k);
            (key, AccessGuard::from(v))
        }))
    }
}
