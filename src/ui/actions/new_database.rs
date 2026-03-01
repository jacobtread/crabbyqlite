use anyhow::Context;
use gpui::{App, actions};
use std::path::PathBuf;
use std::rc::Rc;
use tokio::fs::File;

use crate::database::AnySharedDatabase;
use crate::database::sqlite::SqliteDatabase;
use crate::state::AppStateExt;
use crate::state::async_resource::AsyncResourceEntityExt;
use crate::ui::gpui_tokio::Tokio;

actions!(file, [NewDatabase]);

fn default_directory() -> Option<PathBuf> {
    let user_dirs = directories::UserDirs::new()?;

    Some(user_dirs.home_dir().to_path_buf())
}

pub fn new_database(_: &NewDatabase, cx: &mut App) {
    let document_dir = default_directory().unwrap_or(PathBuf::from("."));

    let prompt_recv = cx.prompt_for_new_path(&document_dir, Some("database.db"));

    let path = Tokio::spawn_result(cx, async move {
        let path = match prompt_recv.await {
            Ok(Ok(Some(value))) => value,

            // Error occurred
            Ok(Err(error)) => {
                tracing::error!(?error, "failed to pick file");
                return Err(error);
            }

            // Cancelled picking the file or picked nothing
            Err(_) | Ok(Ok(None)) => return Ok(None),
        };

        if let Err(error) = File::create(&path).await {
            tracing::error!(?error, "failed to create file");
            return Err(error.into());
        }

        tracing::debug!(?path, "picked file for opening");
        Ok(Some(path))
    });

    let database = cx.database();

    database.maybe_load(cx, async move || {
        let path = match path.await {
            Ok(Some(value)) => value,

            Ok(None) => return Ok(None),

            Err(error) => return Err(error.context("failed to run task")),
        };

        let database = SqliteDatabase::from_path(&path)
            .await
            .context("failed to connect to database")?;
        let database: AnySharedDatabase = Rc::new(database);

        tracing::debug!("loaded database");

        Ok(Some(database))
    });
}
