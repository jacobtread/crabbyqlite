use anyhow::Context;
use gpui::{App, actions};
use std::path::PathBuf;
use std::rc::Rc;
use tokio::fs::File;

use crate::database::AnySharedDatabase;
use crate::database::sqlite::{SqliteDatabase, SqliteDatabaseOptions};
use crate::state::AppStateExt;
use crate::state::async_resource::AsyncResourceEntityExt;
use crate::utils::async_utils::resolve_async_callback_cx;

actions!(file, [NewDatabase]);

fn default_directory() -> Option<PathBuf> {
    let user_dirs = directories::UserDirs::new()?;

    Some(user_dirs.home_dir().to_path_buf())
}

pub fn new_database(_: &NewDatabase, cx: &mut App) {
    let document_dir = default_directory().unwrap_or(PathBuf::from("."));

    let prompt_recv = cx.prompt_for_new_path(&document_dir, Some("database.db"));

    resolve_async_callback_cx(cx, prompt_recv, move |cx, prompt_result| {
        let path = match prompt_result {
            Ok(Ok(Some(value))) => value,

            // Error occurred
            Ok(Err(_error)) => {
                //TODO: REPORT ERROR
                return;
            }

            // Cancelled picking the file or picked nothing
            Err(_) | Ok(Ok(None)) => return,
        };

        on_database_path_picked(cx, path);
    });
}

/// Handle the file `path` of the database being picked
fn on_database_path_picked(cx: &mut App, path: PathBuf) {
    let database = cx.database();

    database.maybe_load(cx, async move || {
        if let Err(error) = File::create(&path).await {
            tracing::error!(?error, "failed to create file");
            return Err(error.into());
        }

        tracing::debug!(?path, "picked file for opening");

        let database = SqliteDatabase::from_path(&path, SqliteDatabaseOptions::default())
            .await
            .context("failed to connect to database")?;
        let database: AnySharedDatabase = Rc::new(database);

        tracing::debug!("loaded database");

        Ok(Some(database))
    });
}
