use gpui::{App, actions};

use crate::state::{async_resource::AsyncResourceEntityExt, database::DatabaseResourceExt};

actions!(file, [CloseDatabase]);

pub fn close_database(_: &CloseDatabase, cx: &mut App) {
    let database = cx.database();
    database.set_idle(cx);

    tracing::debug!("closed database");
}
