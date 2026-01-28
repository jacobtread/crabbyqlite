use gpui::{App, BorrowAppContext, actions};

use crate::state::AppState;

actions!(file, [CloseDatabase]);

pub fn close_database(_: &CloseDatabase, cx: &mut App) {
    cx.update_global(|global: &mut AppState, cx| {
        let database_store = global.database_store.clone();
        database_store.update(cx, |this, cx| {
            this.set_database(None, cx);
        });
    });
}
