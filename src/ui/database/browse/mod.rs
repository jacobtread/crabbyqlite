use gpui::{App, AppContext, Entity, ParentElement, Render, Styled, Window, div};
use gpui_component::{
    ActiveTheme, Icon, StyledExt, alert::Alert, select::SelectEvent, spinner::Spinner,
};

use crate::{
    database::DatabaseTable,
    state::{async_resource::AsyncResource, database_tables::database_tables_resource},
    ui::{
        database::{
            browse::table_browser::DatabaseTableBrowser,
            browse::toolbar::DatabaseBrowseDataViewToolbar,
        },
        icons::CustomIconName,
        translated::ts,
    },
};

mod table_browser;
mod toolbar;

pub struct DatabaseBrowseDataView {
    /// Currently loaded set of database tables
    tables: Entity<AsyncResource<Vec<DatabaseTable>>>,

    /// Toolbar component
    toolbar: Entity<DatabaseBrowseDataViewToolbar>,

    /// Browser for the currently selected table
    browser: Option<Entity<DatabaseTableBrowser>>,
}

impl DatabaseBrowseDataView {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let tables = database_tables_resource(cx);

            let toolbar = DatabaseBrowseDataViewToolbar::new(window, cx);

            // Listen to changes in the tables list to update the table selector values and selection
            cx.observe_in(
                &tables,
                window,
                move |this: &mut DatabaseBrowseDataView, tables, window, cx| {
                    let tables_data = match tables.read(cx) {
                        AsyncResource::Loaded(tables) => tables.clone(),
                        _ => Vec::new(),
                    };

                    // Collect the table items for the selector
                    let tables: Vec<String> =
                        tables_data.iter().map(|value| value.name.clone()).collect();

                    let table = this
                        .toolbar
                        .update(cx, |this, cx| this.update_tables(tables, window, cx));

                    this.browser = table.map(|table| DatabaseTableBrowser::new(table, window, cx));
                },
            )
            .detach();

            // Listen to changes of the current table to update the browser view
            let table_selector_state = toolbar.read(cx).table_select_state();
            cx.subscribe_in(
                &table_selector_state,
                window,
                move |this, _entity, event, window, cx| match event {
                    SelectEvent::Confirm(table) => {
                        if let Some(table) = table {
                            this.browser =
                                Some(DatabaseTableBrowser::new(table.clone(), window, cx));
                        } else {
                            this.browser = None;
                        }
                    }
                },
            )
            .detach();

            Self {
                tables,
                toolbar,
                browser: None,
            }
        })
    }
}

impl Render for DatabaseBrowseDataView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        match self.tables.read(cx) {
            //
            AsyncResource::Idle | AsyncResource::Loading(_) => div()
                .justify_center()
                //
                .child(Spinner::new()),

            //
            AsyncResource::Loaded(_) => div()
                .size_full()
                .v_flex()
                .child(self.toolbar.clone())
                .child(match &self.browser {
                    Some(browser) => div()
                        //
                        .flex_auto()
                        .size_full()
                        .child(browser.clone()),
                    None => div().size_full().v_flex().flex_auto().child(
                        div()
                            .v_flex()
                            .text_left()
                            .p_4()
                            .child(Icon::new(CustomIconName::Cable).size_10())
                            .child(div().child(ts("no-active-database.title")).text_lg())
                            .child(
                                div()
                                    .text_sm()
                                    .child(ts("no-active-database.description"))
                                    .text_color(cx.theme().muted_foreground),
                            ),
                    ),
                }),

            //
            AsyncResource::Error(error) => div()
                .p_3()
                .child(Alert::error("error-alert", error.clone()).title(ts("error"))),
        }
    }
}
