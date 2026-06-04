use crate::{
    database::AnySharedDatabase,
    state::{async_resource::AsyncResource, database::DatabaseResourceExt},
    ui::{
        components::atoms::{
            i18n::translated::{t, ts},
            icons::CustomIconName,
        },
        views::database::{
            browse_table::DatabaseBrowseTableView, browse_tables::DatabaseBrowseTablesView,
            query_executor::DatabaseQueryExecutor,
        },
    },
};
use gpui::{App, AppContext, Context, Entity, ParentElement, Render, Styled, Window, div};
use gpui_component::{
    ActiveTheme, Icon, StyledExt,
    spinner::Spinner,
    tab::{Tab, TabBar},
};

mod browse_table;
mod browse_tables;
mod edit_pragmas;
mod query_executor;

pub struct DatabaseView {
    active_tab: usize,
    tables_view: Entity<DatabaseBrowseTablesView>,
    executor: Entity<DatabaseQueryExecutor>,
    browse_view: Entity<DatabaseBrowseTableView>,
    database: Entity<AsyncResource<AnySharedDatabase>>,
}

impl DatabaseView {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let database = cx.database();

            DatabaseView {
                active_tab: 0,
                tables_view: DatabaseBrowseTablesView::new(cx),
                executor: DatabaseQueryExecutor::new(window, cx),
                browse_view: DatabaseBrowseTableView::new(window, cx),
                database,
            }
        })
    }

    fn on_change_tab(&mut self, index: &usize, _window: &mut Window, cx: &mut Context<Self>) {
        self.active_tab = *index;
        cx.notify();
    }
}

impl Render for DatabaseView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        match self.database.read(cx) {
            AsyncResource::Idle => div().size_full().v_flex().child(
                div()
                    .v_flex()
                    .text_left()
                    .p_4()
                    .child(Icon::new(CustomIconName::Cable).size_10())
                    .child(div().child(t("no-active-database.title")).text_lg())
                    .child(
                        div()
                            .text_sm()
                            .child(t("no-active-database.description"))
                            .text_color(cx.theme().muted_foreground),
                    ),
            ),
            AsyncResource::Loading(_) => div()
                .justify_center()
                //
                .child(ts("loading-database"))
                .child(Spinner::new()),
            AsyncResource::Loaded(_database) => div()
                .size_full()
                .v_flex()
                .child(
                    TabBar::new("tabs")
                        .selected_index(self.active_tab)
                        .on_click(cx.listener(Self::on_change_tab))
                        .child(Tab::new().label("Database Structure"))
                        .child(Tab::new().label("Browse Data"))
                        .child(Tab::new().label("Edit Pragmas"))
                        .child(Tab::new().label("Execute SQL")),
                )
                .child(match self.active_tab {
                    0 => div().size_full().child(self.tables_view.clone()),
                    1 => div().size_full().child(self.browse_view.clone()),
                    2 => div().child("Edit Pragmas is not available yet"),
                    3 => div().size_full().child(self.executor.clone()),
                    _ => div(),
                }),
            AsyncResource::Error(error) => div().child("TODO: Error message").child(error.clone()),
        }
    }
}
