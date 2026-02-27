use gpui::{App, actions};

use crate::state::{AppStateExt, async_resource::AsyncResourceEntityExt};

actions!(file, [CloseDatabase]);

pub fn close_database(_: &CloseDatabase, cx: &mut App) {
    let database = cx.database();
    database.set_idle(cx);
}
