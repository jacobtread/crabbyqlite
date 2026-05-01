use std::{path::PathBuf, rc::Rc};

use crate::{
    database::{
        AnySharedDatabase,
        sqlite::{SqliteDatabase, SqliteDatabaseOptions},
    },
    state::{AppStateExt, async_resource::AsyncResourceEntityExt},
    utils::async_utils::resolve_async_callback_cx,
};
use anyhow::Context;
use gpui::{Action, App, AppContext, ParentElement, PathPromptOptions, Styled, Window, div};
use gpui_component::{
    StyledExt, WindowExt,
    button::{Button, ButtonVariants},
    input::{Input, InputState},
};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = file)]
pub struct OpenFileEncrypted {
    pub read_only: bool,
}

pub fn open_encrypted_database(OpenFileEncrypted { read_only }: &OpenFileEncrypted, cx: &mut App) {
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

        on_encrypted_database_path_picked(cx, path, readonly);
    });
}

/// Handle the user completing picking the database path
fn on_encrypted_database_path_picked(cx: &mut App, path: PathBuf, readonly: bool) {
    let window = cx.active_window().expect("expected a active window");
    _ = window.update(cx, move |_view, window, cx| {
        on_open_password_dialog(window, cx, path, readonly);
    });
}

/// Open a dialog to prompt for the password for the database at `path`
/// optionally opening as `readonly`
fn on_open_password_dialog(window: &mut Window, cx: &mut App, path: PathBuf, readonly: bool) {
    let input = cx.new(|cx| {
        InputState::new(window, cx)
            .masked(true)
            .placeholder("Database key...")
    });

    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title("Database Key")
            .child(
                div()
                    .v_flex()
                    .gap_3()
                    .child("Please enter your details:")
                    .child(Input::new(&input)),
            )
            .footer(
                div()
                    .h_flex()
                    .child(Button::new("ok").primary().label("Submit").on_click({
                        let path = path.clone();
                        let input = input.clone();

                        move |_, window, cx| {
                            let path = path.clone();
                            let key = input.read(cx).value().to_string();
                            window.close_dialog(cx);

                            on_database_password(cx, path, readonly, key);
                        }
                    }))
                    .child(
                        Button::new("cancel")
                            .label("Cancel")
                            .on_click(|_, window, cx| {
                                window.close_dialog(cx);
                            }),
                    ),
            )
    });
}

/// Handle the password (`key`) being provided for the database at `path` attempts
/// to connect to and use the database optionally opening as `readonly`
fn on_database_password(cx: &mut App, path: PathBuf, readonly: bool, key: String) {
    let database = cx.database();

    database.maybe_load(cx, async move || {
        tracing::debug!(?path, "picked file for opening");

        let options = SqliteDatabaseOptions {
            readonly,
            key: Some(key),
        };

        let database = SqliteDatabase::from_path(&path, options)
            .await
            .context("failed to connect to database")?;
        let database: AnySharedDatabase = Rc::new(database);

        tracing::debug!("loaded database");

        Ok(Some(database))
    });
}
