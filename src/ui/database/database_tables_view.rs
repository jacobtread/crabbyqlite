use gpui::{App, AppContext, Entity, ParentElement, Render, Styled, Window, div};
use gpui_component::{alert::Alert, spinner::Spinner};

use crate::{
    database::DatabaseTable,
    state::{async_resource::AsyncResource, database_tables::database_tables_resource},
    ui::{database::tables::DatabaseTablesTreeView, translated::ts},
};

pub struct DatabaseTablesView {
    /// Currently loaded set of database tables
    tables: Entity<AsyncResource<Vec<DatabaseTable>>>,

    tree: Entity<DatabaseTablesTreeView>,
}

impl DatabaseTablesView {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let tree = DatabaseTablesTreeView::new(window, cx);

        cx.new(|cx| {
            let tables = database_tables_resource(cx);

            cx.observe(&tables, |this: &mut DatabaseTablesView, tables, cx| {
                let tables_data = match tables.read(cx) {
                    AsyncResource::Loaded(tables) => tables.clone(),
                    _ => Vec::new(),
                };

                tracing::debug!("loaded tables data for display");

                this.tree.update(cx, |this, cx| {
                    this.set_entries(tables_data, cx);
                });
            })
            .detach();

            Self { tables, tree }
        })
    }
}

impl Render for DatabaseTablesView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        match self.tables.read(cx) {
            //
            AsyncResource::Idle | AsyncResource::Loading(_) => div()
                .size_full()
                .justify_center()
                //
                .child(Spinner::new()),

            //
            AsyncResource::Loaded(_) => div()
                .size_full()
                //
                .child(self.tree.clone()),

            //
            AsyncResource::Error(error) => div()
                .p_3()
                .child(Alert::error("error-alert", error.clone()).title(ts("error"))),
        }
    }
}
