use crate::{
    database::{AnySharedDatabase, sqlite::SqliteDatabase},
    state::{AppStateExt, async_resource::AsyncResourceEntityExt},
};
use anyhow::Context;
use gpui::{Action, App, PathPromptOptions};
use schemars::JsonSchema;
use serde::Deserialize;
use std::rc::Rc;

#[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = file)]
pub struct OpenFile {
    pub read_only: bool,
}

pub fn open_file(OpenFile { read_only }: &OpenFile, cx: &mut App) {
    let prompt_recv = cx.prompt_for_paths(PathPromptOptions {
        files: true,
        multiple: false,
        directories: false,
        prompt: Some("SQLite database files (*.db, *.sqlite, *.sqlite3, *.db3)".into()),
    });

    let database = cx.database();
    let read_only = *read_only;

    database.maybe_load(cx, async move || {
        let paths = match prompt_recv.await {
            Ok(Ok(Some(value))) => value,

            // Error occurred
            Ok(Err(error)) => return Err(error.context("failed to pick file")),

            // Cancelled picking the file or picked nothing
            Err(_) | Ok(Ok(None)) => return Ok(None),
        };

        let path = match paths.first() {
            Some(value) => value,
            // Picked nothing
            None => return Ok(None),
        };

        tracing::debug!(?path, "picked file for opening");

        let database = SqliteDatabase::from_path(path, read_only)
            .await
            .context("failed to connect to database")?;
        let database: AnySharedDatabase = Rc::new(database);

        tracing::debug!("loaded database");

        Ok(Some(database))
    });
}
