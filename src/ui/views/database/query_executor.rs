use crate::{
    database::{AnySharedDatabase, DatabaseQueryResult, DatabaseRow},
    state::{
        async_resource::{AsyncResource, AsyncResourceEntityExt},
        database::{DatabaseResourceExt, connection::QueryExecutedEvent},
    },
    ui::components::{atoms::i18n::translated::ts, organisms::sql_editor::SqlEditor},
};
use anyhow::Context as AnyhowContext;
use gpui::{
    App, AppContext, ClickEvent, Context, Entity, IntoElement, ParentElement, Render, SharedString,
    Styled, Subscription, Window, div,
};
use gpui_component::{
    Sizable, StyledExt,
    alert::Alert,
    button::Button,
    input::{InputEvent, InputState},
    resizable::v_resizable,
    spinner::Spinner,
    table::{Column, DataTable, TableDelegate, TableState},
};
use sqlformat::FormatOptions;

pub struct DatabaseQueryExecutor {
    /// Query results
    results: Entity<AsyncResource<DatabaseQueryResult>>,

    /// State for the results table
    table_state: Entity<TableState<ResultsTableDelegate>>,

    // SQL Editor state
    editor: Entity<SqlEditor>,

    _subscriptions: (Subscription, Subscription, Subscription),
}

struct ResultsTableDelegate {
    rows: Vec<DatabaseRow>,
    columns: Vec<Column>,
}

impl ResultsTableDelegate {
    fn new() -> Self {
        Self {
            rows: vec![],
            columns: vec![],
        }
    }
}

impl TableDelegate for ResultsTableDelegate {
    fn columns_count(&self, _: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.rows.len()
    }

    fn column(&self, col_ix: usize, _: &App) -> Column {
        self.columns[col_ix].clone()
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let row = &self.rows[row_ix];
        let value = row.values.get(col_ix);
        value.cloned().unwrap_or_default()
    }
}

impl DatabaseQueryExecutor {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let table_delegate = ResultsTableDelegate::new();
        let table_state = cx.new(|cx| TableState::new(table_delegate, window, cx));
        let database = cx.database();

        let editor = SqlEditor::new(window, cx, "".into(), false, true, database);

        cx.new(|cx| {
            let results: Entity<AsyncResource<DatabaseQueryResult>> = AsyncResource::new(cx);
            let database = cx.database();

            let tables_subscription = cx.observe(&database, Self::on_tables_changed);

            // Observe results changes to update the database table
            let results_subscription = cx.observe(&results, Self::on_results_changed);

            // Handle CTRL + Enter to run the query
            let editor_input_state = editor.read(cx).input_state.clone();
            let editor_input_subscription =
                cx.subscribe_in(&editor_input_state, window, Self::on_editor_input);

            Self {
                results,
                table_state,
                editor,
                _subscriptions: (
                    tables_subscription,
                    results_subscription,
                    editor_input_subscription,
                ),
            }
        })
    }

    /// Handles changes to the database connection
    fn on_tables_changed(
        &mut self,
        _database: Entity<AsyncResource<AnySharedDatabase>>,
        cx: &mut Context<Self>,
    ) {
        // Reset the results resource when the database changes
        self.results.set_idle(cx);
    }

    /// Handles changes to the query results
    fn on_results_changed(
        &mut self,
        results: Entity<AsyncResource<DatabaseQueryResult>>,
        cx: &mut Context<Self>,
    ) {
        let (rows, columns) = match results.read(cx) {
            AsyncResource::Loaded(rows) => (rows.rows.clone(), rows.column_names.clone()),
            _ => (Vec::new(), Vec::new()),
        };

        self.update_result_rows(rows, columns, cx);
    }

    fn on_editor_input(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let InputEvent::PressEnter { secondary, shift } = event {
            if *secondary {
                self.perform_current_query(cx);
            }
            // Since we enable the submit on enter behavior in order to make CTRL + ENTER not
            // make a new line we need to mick the new line behavior when shift isn't held
            else if !shift {
                state.update(cx, |state, cx| {
                    state.insert("\n", window, cx);
                });
            }
        }
    }

    fn update_result_rows(
        &mut self,
        rows: Vec<DatabaseRow>,
        columns: Vec<SharedString>,
        cx: &mut Context<'_, Self>,
    ) {
        self.table_state.update(cx, |this, cx| {
            let delegate = this.delegate_mut();

            delegate.columns = columns
                .into_iter()
                .map(|column| Column::new(column.clone(), column))
                .collect();

            delegate.rows = rows;

            this.refresh(cx);
        });
    }

    fn perform_current_query(&mut self, cx: &mut Context<'_, Self>) {
        let editor = self.editor.read(cx);
        let query = editor.input_state.read(cx).value();
        self.perform_query(cx, query);
    }

    fn perform_query(&mut self, cx: &mut Context<'_, Self>, query: SharedString) {
        let database_entity = cx.database_connection_resource();
        let database = match cx.database_connection() {
            Some(value) => value,
            None => return,
        };

        // Notify our listeners that we are executing a query
        database_entity.update(cx, |_, cx| {
            cx.emit(QueryExecutedEvent {
                query: query.clone(),
            });
        });

        self.results.load(cx, async move || {
            database
                .query(query.as_ref())
                .await
                .context("failed to execute query")
        });
    }

    fn on_executor(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.perform_current_query(cx);
    }

    fn on_format_sql(&mut self, _event: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let editor = self.editor.read(cx);
        let input_state = editor.input_state.clone();

        let query = input_state.read(cx).value();
        let options = FormatOptions::default();
        let formatted = sqlformat::format(&query, &sqlformat::QueryParams::None, &options);

        input_state.update(cx, move |this, cx| {
            this.set_value(formatted, window, cx);
        });
    }
}

impl Render for DatabaseQueryExecutor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div().size_full().child(
            v_resizable("executor-resizable")
                .child(
                    div()
                        .v_flex()
                        .w_full()
                        .h_full()
                        .child(
                            div()
                                .h_flex()
                                .child(
                                    Button::new("execute")
                                        .child(ts("execute"))
                                        .small()
                                        .on_click(cx.listener(Self::on_executor)),
                                )
                                .child(
                                    Button::new("format")
                                        .child(ts("format"))
                                        .small()
                                        .on_click(cx.listener(Self::on_format_sql)),
                                ),
                        )
                        .child(self.editor.clone())
                        .into_any_element(),
                )
                .child(
                    match self.results.read(cx) {
                        AsyncResource::Idle => div()
                            .size_full()
                            .p_3()
                            .text_sm()
                            .child("Query results will appear here"),
                        AsyncResource::Loading(_) => div()
                            .size_full()
                            .justify_center()
                            //
                            .child(Spinner::new()),
                        AsyncResource::Loaded(_) => div()
                            .size_full()
                            //
                            .child(
                                DataTable::new(&self.table_state)
                                    .stripe(true)
                                    .bordered(true)
                                    .scrollbar_visible(true, true),
                            ),
                        AsyncResource::Error(error) => div()
                            .p_3()
                            .child(Alert::error("error-alert", error.clone()).title(ts("error"))),
                    }
                    .into_any_element(),
                ),
        )
    }
}
