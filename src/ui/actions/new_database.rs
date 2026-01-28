use gpui::{App, actions};
use std::{path::PathBuf, sync::Arc};
use tokio::fs::File;

use crate::ui::gpui_tokio::Tokio;
use crate::{database::sqlite::SqliteDatabase, state::AppState};

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

    cx.spawn(async move |cx| {
        let path = match path.await {
            Ok(Some(value)) => value,

            Ok(None) => return,

            Err(error) => {
                tracing::error!(?error, "failed to run task");
                return;
            }
        };

        let database = match SqliteDatabase::from_path(&path).await {
            Ok(value) => value,
            Err(error) => {
                tracing::error!(?error, "failed to connect to database");
                return;
            }
        };

        tracing::debug!("loaded database");

        if let Err(error) = cx.update_global(|global: &mut AppState, cx| {
            let database_store = global.database_store.clone();
            database_store.update(cx, |this, cx| {
                this.set_database(Some(Arc::new(database)), cx);
            })
        }) {
            tracing::error!(?error, "failed to update global state")
        }
    })
    .detach();
}
