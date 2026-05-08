pub mod base;
#[cfg(feature = "persistence")]
pub mod redb_indexstore;

pub use base::{IndexStore, IndexStruct, SimpleIndexStore};
#[cfg(feature = "persistence")]
pub use redb_indexstore::RedbIndexStore;
