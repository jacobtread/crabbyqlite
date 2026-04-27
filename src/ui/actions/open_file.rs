use crate::{
    database::{
        AnySharedDatabase,
        sqlite::{SqliteDatabase, SqliteDatabaseOptions},
    },
    state::{AppStateExt, async_resource::AsyncResourceEntityExt},
    utils::async_utils::resolve_async_callback_cx,
};
use anyhow::Context;
use gpui::{Action, App, PathPromptOptions};
use schemars::JsonSchema;
use serde::Deserialize;
use std::{path::PathBuf, rc::Rc};

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

    let readonly = *read_only;

    resolve_async_callback_cx(cx, prompt_recv, move |cx, prompt_result| {
        let paths = match prompt_result {
            Ok(Ok(Some(value))) => value,

            // Error occurred
            Ok(Err(_error)) => {
                //TODO: REPORT ERROR
                return;
            }

            // Cancelled picking the file or picked nothing
            Err(_) | Ok(Ok(None)) => return,
        };

        let path = match paths.first() {
            Some(value) => value,
            // Picked nothing
            None => return,
        };

        let path = path.to_path_buf();

        on_database_path_picked(cx, path, readonly);
    });
}

/// Handle the file `path` of the database being picked
fn on_database_path_picked(cx: &mut App, path: PathBuf, readonly: bool) {
    let database = cx.database();
    database.maybe_load(cx, async move || {
        tracing::debug!(?path, "picked file for opening");

        let options = SqliteDatabaseOptions {
            readonly,
            ..Default::default()
        };

        let database = SqliteDatabase::from_path(&path, options)
            .await
            .context("failed to connect to database")?;
        let database: AnySharedDatabase = Rc::new(database);

        tracing::debug!("loaded database");

        Ok(Some(database))
    });
}
