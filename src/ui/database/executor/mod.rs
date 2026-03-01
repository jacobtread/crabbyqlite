use crate::{
    database::DatabaseRow,
    state::{
        AppStateExt,
        async_resource::{AsyncResource, AsyncResourceEntityExt},
    },
    ui::{sql_editor::SqlEditor, translated::ts},
};
use anyhow::Context;
use gpui::{
    App, AppContext, Entity, IntoElement, ParentElement, Render, SharedString, Styled, Window, div,
};
use gpui_component::{
    Sizable, StyledExt,
    alert::Alert,
    button::Button,
    resizable::v_resizable,
    spinner::Spinner,
    table::{Column, DataTable, TableDelegate, TableState},
};
use sqlformat::FormatOptions;

pub struct DatabaseSqlExecutor {
    /// Query results
    results: Entity<AsyncResource<Vec<DatabaseRow>>>,

    /// State for the results table
    table_state: Entity<TableState<ResultsTableDelegate>>,

    // SQL Editor state
    editor: Entity<SqlEditor>,
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

    fn column(&self, col_ix: usize, _: &App) -> Column {
        self.columns[col_ix].clone()
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut gpui::Context<TableState<Self>>,
    ) -> impl IntoElement {
        let row = &self.data[row_ix];
        let value = row.values.get(col_ix);
        value.cloned().unwrap_or_default()
    }
}

impl DatabaseSqlExecutor {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let table_delegate = ResultsTableDelegate::new();
        let table_state = cx.new(|cx| TableState::new(table_delegate, window, cx));
        let database = cx.database();

        let editor = SqlEditor::new(window, cx, "".into(), false, database);

        cx.new(|cx| {
            let results: Entity<AsyncResource<Vec<DatabaseRow>>> = AsyncResource::new(cx);
            let database = cx.database();

            // Reset the results resource when the database changes
            cx.observe(
                &database,
                |this: &mut DatabaseSqlExecutor, _database, cx| {
                    this.results.set_idle(cx);
                },
            )
            .detach();

            // Observe results changes to update the database table
            cx.observe(&results, |this: &mut DatabaseSqlExecutor, results, cx| {
                let rows = match results.read(cx) {
                    AsyncResource::Loaded(rows) => rows.clone(),
                    _ => Vec::new(),
                };

                this.update_result_rows(rows, cx);
            })
            .detach();

            Self {
                results,
                table_state,
                editor,
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

    fn perform_query(&mut self, cx: &mut gpui::Context<'_, Self>, query: SharedString) {
        let database = match cx.current_database() {
            Some(value) => value,
            None => return,
        };

        self.results.load(cx, async move || {
            database
                .query(query.as_ref())
                .await
                .context("failed to execute query")
        });
    }
}

impl Render for DatabaseSqlExecutor {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
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
                                        .on_click(cx.listener(|this, _event, _window, cx| {
                                            let editor = this.editor.read(cx);
                                            let query = editor.input_state.read(cx).value();
                                            this.perform_query(cx, query);
                                        })),
                                )
                                .child(Button::new("format").child(ts("format")).small().on_click(
                                    cx.listener(|this, _event, window, cx| {
                                        let editor = this.editor.read(cx);
                                        let input_state = editor.input_state.clone();

                                        let query = input_state.read(cx).value();
                                        let options = FormatOptions::default();
                                        let formatted = sqlformat::format(
                                            &query,
                                            &sqlformat::QueryParams::None,
                                            &options,
                                        );

                                        input_state.update(cx, move |this, cx| {
                                            this.set_value(formatted, window, cx);
                                        });
                                    }),
                                )),
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
