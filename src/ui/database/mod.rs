use crate::{
    database::AnySharedDatabase,
    state::{AppStateExt, async_resource::AsyncResource},
    ui::{
        database::{
            browse::DatabaseBrowseDataView, database_sql_executor::DatabaseSqlExecutor,
            database_tables_view::DatabaseTablesView,
        },
        icons::CustomIconName,
        translated::{t, ts},
    },
};
use gpui::{App, AppContext, Element, Entity, ParentElement, Render, Styled, Window, div};
use gpui_component::{
    ActiveTheme, Icon, StyledExt,
    spinner::Spinner,
    tab::{Tab, TabBar},
};

mod browse;
mod database_sql_editor;
mod database_sql_executor;
mod database_tables_view;

pub struct DatabaseView {
    active_tab: usize,
    tables_view: Entity<DatabaseTablesView>,
    sql_editor: Entity<DatabaseSqlExecutor>,
    browse_view: Entity<DatabaseBrowseDataView>,
    database: Entity<AsyncResource<AnySharedDatabase>>,
}

impl DatabaseView {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let database = cx.database();

            DatabaseView {
                active_tab: 0,
                tables_view: DatabaseTablesView::new(window, cx),
                sql_editor: DatabaseSqlExecutor::new(window, cx),
                browse_view: DatabaseBrowseDataView::new(window, cx),
                database,
            }
        })
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
                        .on_click(cx.listener(|view, index, _, cx| {
                            view.active_tab = *index;
                            cx.notify();
                        }))
                        .child(Tab::new().label("Database Structure"))
                        .child(Tab::new().label("Browse Data"))
                        .child(Tab::new().label("Edit Pragmas"))
                        .child(Tab::new().label("Execute SQL")),
                )
                .child(match self.active_tab {
                    0 => div().size_full().child(self.tables_view.clone()).into_any(),
                    1 => div().size_full().child(self.browse_view.clone()).into_any(),
                    2 => div().child("Edit Pragmas").into_any(),
                    3 => div().size_full().child(self.sql_editor.clone()).into_any(),
                    _ => div().into_any(),
                }),
            AsyncResource::Error(error) => div().child("TODO: Error message").child(error.clone()),
        }
    }
}
