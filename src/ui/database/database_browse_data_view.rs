use std::sync::Arc;

use gpui::{
    App, AppContext, Context, Element, ElementId, Entity, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};
use gpui_component::{
    ActiveTheme, StyledExt, h_flex,
    select::{SearchableVec, Select, SelectEvent, SelectState},
    spinner::Spinner,
    table::{Column, Table, TableDelegate, TableState},
    tooltip::Tooltip,
};

use crate::{
    database::{DatabaseRow, DatabaseTable, DatabaseTableQuery},
    state::{
        AnySharedDatabase, AppState, DatabaseStore,
        async_resource::{AsyncResource, AsyncResourceEntityExt},
        database_tables::database_tables_resource,
    },
};

pub struct DatabaseBrowseDataView {
    /// Currently loaded set of database tables
    tables: Entity<AsyncResource<Vec<DatabaseTable>>>,

    /// Data for the currently viewed table
    active_table_data: Entity<AsyncResource<ActiveTableData>>,

    /// State for the table selector drop down
    table_selector_state: Entity<SelectState<SearchableVec<String>>>,

    /// State for the results table
    table_state: Entity<TableState<ResultsTableDelegate>>,
}

struct QueryResultRow {
    values: Vec<SharedString>,
}

struct ResultsTableDelegate {
    data: Vec<QueryResultRow>,
    columns: Vec<Column>,
}

fn compute_columns(rows: &[DatabaseRow]) -> Vec<Column> {
    let first_row = match rows.first() {
        Some(value) => value,
        None => return Vec::new(),
    };

    first_row
        .value
        .iter()
        .map(|value| Column::new(value.name.clone(), value.name.clone()))
        .collect()
}

impl ResultsTableDelegate {
    fn new() -> Self {
        Self {
            data: vec![],
            columns: vec![],
        }
    }
}

impl TableDelegate for ResultsTableDelegate {
    fn columns_count(&self, _: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.data.len()
    }

    fn column(&self, col_ix: usize, _: &App) -> &Column {
        &self.columns[col_ix]
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let row = &self.data[row_ix];
        let value = row.values.get(col_ix);
        value.cloned().unwrap_or_default()
    }
}

#[derive(Clone)]
pub struct ActiveTableData {
    rows: Vec<DatabaseRow>,
    count: i64,
}

impl DatabaseBrowseDataView {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let table_delegate = ResultsTableDelegate::new();
        let table_state = cx.new(|cx| TableState::new(table_delegate, window, cx));

        cx.new(|cx| {
            let tables = database_tables_resource(window, cx);
            let active_table_data = AsyncResource::<ActiveTableData>::new(cx);

            let app = cx.global::<AppState>();
            let database_store = app.database_store.clone();

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

                    let app = cx.global::<AppState>();
                    let database_store = app.database_store.clone();
                    this.query_table_data(&database_store, window, cx);
                },
            )
            .detach();

            cx.observe(
                &active_table_data,
                |this: &mut DatabaseBrowseDataView, tables, cx| {
                    let rows = match tables.read(cx) {
                        AsyncResource::Loaded(tables) => tables.rows.clone(),
                        _ => Vec::new(),
                    };

                    tracing::debug!("setting rows: {}", rows.len());

                    this.update_result_rows(rows, cx);
                },
            )
            .detach();

            let table_selector_state =
                cx.new(|cx| SelectState::new(SearchableVec::new(Vec::new()), None, window, cx));

            cx.subscribe_in(
                &table_selector_state,
                window,
                move |view, state, event, window, cx| match event {
                    SelectEvent::Confirm(value) => {
                        view.query_table_data(&database_store, window, cx);
                    }
                },
            )
            .detach();

            Self {
                tables,
                active_table_data,
                table_selector_state,
                table_state,
            }
        })
    }

    fn update_result_rows(&mut self, rows: Vec<DatabaseRow>, cx: &mut gpui::Context<'_, Self>) {
        self.table_state.update(cx, |this, cx| {
            let delegate = this.delegate_mut();

            delegate.columns = compute_columns(&rows);
            delegate.data = rows
                .into_iter()
                .map(|table| QueryResultRow {
                    values: table
                        .value
                        .into_iter()
                        .map(|value| value.value.into())
                        .collect(),
                })
                .collect();

            this.refresh(cx);
        });
    }

    async fn load_active_table_data(
        database: AnySharedDatabase,
        table: String,
    ) -> anyhow::Result<ActiveTableData> {
        let limit = 50;
        let offset = 0;

        let query = DatabaseTableQuery { table };

        let rows = database
            .query_table_rows(query.clone(), limit, offset)
            .await?;
        let count = database.query_table_rows_count(query.clone()).await?;

        tracing::debug!(?count, ?rows, "loaded data");

        Ok(ActiveTableData { rows, count })
    }

    fn query_table_data(
        &mut self,
        database_store: &Entity<DatabaseStore>,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<'_, Self>,
    ) {
        let database_store = database_store.read(cx);
        let database = match database_store.database.as_ref() {
            Some(value) => value.clone(),
            None => {
                self.active_table_data.set_idle(cx);
                return;
            }
        };

        let table = match self.table_selector_state.read(cx).selected_value() {
            Some(value) => value.clone(),
            None => {
                self.active_table_data.set_idle(cx);
                return;
            }
        };

        self.active_table_data.load(window, cx, move || {
            Self::load_active_table_data(database, table)
        });
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
            .child(
                div()
                    .flex_auto()
                    .size_full()
                    .child(match self.active_table_data.read(cx) {
                        //
                        AsyncResource::Idle | AsyncResource::Loading(_) => div()
                            .size_full()
                            .justify_center()
                            //
                            .child(Spinner::new()),

                        //
                        AsyncResource::Loaded(_) => div().size_full().child(
                            Table::new(&self.table_state)
                                .stripe(true)
                                .bordered(true)
                                .scrollbar_visible(true, true),
                        ),

                        // TODO: Proper error message display
                        AsyncResource::Error(error) => {
                            div().child("TODO: Error message").child(error.clone())
                        }
                    }),
            )
    }
}
