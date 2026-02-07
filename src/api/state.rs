//! Shared application state for the API.

use crate::db::DbPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DbPool>,
}

impl AppState {
    pub fn new(db: DbPool) -> Self {
        Self { db: Arc::new(db) }
    }
}
