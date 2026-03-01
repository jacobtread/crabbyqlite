use std::rc::Rc;

use anyhow::Context;
use gpui::{App, actions};

use crate::{
    database::{AnySharedDatabase, sqlite::SqliteDatabase},
    state::{AppStateExt, async_resource::AsyncResourceEntityExt},
};

actions!(file, [NewMemoryDatabase]);

pub fn new_memory_database(_: &NewMemoryDatabase, cx: &mut App) {
    let database = cx.database();

    database.load(cx, async move || {
        let database = SqliteDatabase::memory()
            .await
            .context("failed to connect to database")?;
        let database: AnySharedDatabase = Rc::new(database);

        tracing::debug!("loaded database");
        Ok(database)
    });
}
