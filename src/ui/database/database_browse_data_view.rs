use gpui::{App, AppContext, Entity, ParentElement, Render, Styled, Window, div};
use gpui_component::{
    ActiveTheme, StyledExt, h_flex,
    select::{SearchableVec, Select, SelectEvent, SelectState},
    spinner::Spinner,
};

use crate::{
    database::DatabaseTable,
    state::{async_resource::AsyncResource, database_tables::database_tables_resource},
    ui::database::database_table_browser::DatabaseTableBrowser,
};

pub struct DatabaseBrowseDataView {
    /// Currently loaded set of database tables
    tables: Entity<AsyncResource<Vec<DatabaseTable>>>,

    /// State for the table selector drop down
    table_selector_state: Entity<SelectState<SearchableVec<String>>>,

    /// Browser for the currently selected table
    browser: Option<Entity<DatabaseTableBrowser>>,
}

impl DatabaseBrowseDataView {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let tables = database_tables_resource(window, cx);

            cx.observe_in(
                &tables,
                window,
                |this: &mut DatabaseBrowseDataView, tables, window, cx| {
                    let tables_data = match tables.read(cx) {
                        AsyncResource::Loaded(tables) => tables.clone(),
                        _ => Vec::new(),
                    };

                    // Collect the table items for the selector
                    let table_items: Vec<String> =
                        tables_data.iter().map(|value| value.name.clone()).collect();

                    this.table_selector_state.update(cx, |this, cx| {
                        this.set_items(SearchableVec::new(table_items), window, cx);

                        let selected_value = this.selected_value();

                        // Don't change the active table if one is already selected that is in
                        // the new list of available tables
                        if selected_value.is_some_and(|active_table| {
                            tables_data.iter().any(|table| table.name.eq(active_table))
                        }) {
                            return;
                        }

                        let table = tables_data.first();
                        let new_value = table.map(|value| &value.name);
                        if let Some(new_value) = new_value {
                            this.set_selected_value(new_value, window, cx);
                        } else {
                            this.set_selected_index(None, window, cx);
                        }
                    });

                    if let Some(table) = this.table_selector_state.read(cx).selected_value() {
                        this.browser = Some(DatabaseTableBrowser::new(table.clone(), window, cx));
                    } else {
                        this.browser = None;
                    }
                },
            )
            .detach();

            let table_selector_state = cx.new(|cx| {
                SelectState::new(SearchableVec::<String>::new(Vec::new()), None, window, cx)
            });

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
                table_selector_state,
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
        div()
            .size_full()
            .v_flex()
            .child(
                div().h_flex().child(match self.tables.read(cx) {
                    //
                    AsyncResource::Idle | AsyncResource::Loading(_) => div()
                        .justify_center()
                        //
                        .child(Spinner::new()),

                    //
                    AsyncResource::Loaded(_) => div().w_auto().child(
                        Select::new(&self.table_selector_state).empty(
                            h_flex()
                                .justify_center()
                                .text_color(cx.theme().muted_foreground)
                                .child("No options available"),
                        ),
                    ),

                    // TODO: Proper error message display
                    AsyncResource::Error(error) => {
                        div().child("TODO: Error message").child(error.clone())
                    }
                }),
            )
            .child(match &self.browser {
                Some(browser) => div()
                    //
                    .flex_auto()
                    .size_full()
                    .child(browser.clone()),
                None => div()
                    .size_full()
                    .flex_auto()
                    //
                    .child("No table selected"),
            })
    }
}
