use crate::{
    database::DatabaseTable,
    state::{async_resource::AsyncResource, database::DatabaseResourceExt},
    ui::components::{
        atoms::i18n::translated::ts,
        organisms::database::tables_browser::tables_table_tree::DatabaseTablesTreeView,
    },
};
use gpui::{App, AppContext, Context, Entity, ParentElement, Render, Styled, Subscription, div};
use gpui_component::{alert::Alert, spinner::Spinner};

/// View for showing the tables available in the current database
pub struct DatabaseBrowseTablesView {
    tables: Entity<AsyncResource<Vec<DatabaseTable>>>,
    tree: Entity<DatabaseTablesTreeView>,
    _subscriptions: (Subscription,),
}

impl DatabaseBrowseTablesView {
    pub fn new(cx: &mut App) -> Entity<Self> {
        let tree = DatabaseTablesTreeView::new(cx);

        cx.new(|cx| {
            let tables = cx.database_tables();
            let tables_subscription = cx.observe(&tables, Self::on_tables_changed);

            Self {
                tables,
                tree,
                _subscriptions: (tables_subscription,),
            }
        })
    }

    /// Handles changes to the available tables
    fn on_tables_changed(
        &mut self,
        tables: Entity<AsyncResource<Vec<DatabaseTable>>>,
        cx: &mut Context<Self>,
    ) {
        let tables_data = match tables.read(cx) {
            AsyncResource::Loaded(tables) => tables.clone(),
            _ => Vec::new(),
        };

        tracing::debug!("loaded tables data for display");

        self.tree.update(cx, |this, cx| {
            this.set_entries(tables_data, cx);
        });
    }
}

impl Render for DatabaseBrowseTablesView {
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
