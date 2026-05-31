use gpui::{
    App, AppContext, Context, Entity, ParentElement, Render, Styled, Subscription, Window, div,
};
use gpui_component::{
    ActiveTheme, Icon, StyledExt,
    alert::Alert,
    select::{SearchableVec, SelectEvent, SelectState},
    spinner::Spinner,
};

use crate::{
    database::DatabaseTable,
    state::{async_resource::AsyncResource, database::DatabaseResourceExt},
    ui::components::{
        atoms::{i18n::translated::ts, icons::CustomIconName},
        organisms::database::table_browser::{
            table::DatabaseTableBrowser, toolbar::DatabaseBrowseDataViewToolbar,
        },
    },
};

pub struct DatabaseBrowseTableView {
    /// Currently loaded set of database tables
    tables: Entity<AsyncResource<Vec<DatabaseTable>>>,

    /// Toolbar component
    toolbar: Entity<DatabaseBrowseDataViewToolbar>,

    /// Browser for the currently selected table
    browser: Option<Entity<DatabaseTableBrowser>>,

    /// Data subscriptions attached to this view
    _subscriptions: (Subscription, Subscription),
}

impl DatabaseBrowseTableView {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let tables = cx.database_tables();

            let toolbar = DatabaseBrowseDataViewToolbar::new(window, cx);

            // Listen to changes in the tables list to update the table selector values and selection
            let tables_subscription = cx.observe_in(&tables, window, Self::on_tables_changed);

            // Listen to changes of the current table to update the browser view
            let table_selector_state = toolbar.read(cx).table_select_state();
            let table_selector_subscription = cx.subscribe_in(
                &table_selector_state,
                window,
                Self::on_table_selection_changed,
            );

            Self {
                tables,
                toolbar,
                browser: None,
                _subscriptions: (tables_subscription, table_selector_subscription),
            }
        })
    }

    /// Handles changes to the available tables for selection
    fn on_tables_changed(
        &mut self,
        tables: Entity<AsyncResource<Vec<DatabaseTable>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let tables_data = match tables.read(cx) {
            AsyncResource::Loaded(tables) => tables.clone(),
            _ => Vec::new(),
        };

        // Collect the table items for the selector
        let tables: Vec<String> = tables_data.iter().map(|value| value.name.clone()).collect();

        let table = self
            .toolbar
            .update(cx, |this, cx| this.update_tables(tables, window, cx));

        self.browser = table.map(|table| DatabaseTableBrowser::new(table, window, cx));
    }

    /// Handles changes to the currently selected table
    fn on_table_selection_changed(
        &mut self,
        _entity: &Entity<SelectState<SearchableVec<String>>>,
        event: &SelectEvent<SearchableVec<String>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let SelectEvent::Confirm(table) = event;

        self.browser = table
            .clone()
            .map(|table| DatabaseTableBrowser::new(table, window, cx));
    }
}

impl Render for DatabaseBrowseTableView {
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
