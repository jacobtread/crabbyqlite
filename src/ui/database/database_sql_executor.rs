use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled,
    Task, Window, div,
};
use gpui_component::{
    Sizable, StyledExt,
    button::Button,
    spinner::Spinner,
    table::{Column, Table, TableDelegate, TableState},
};

use crate::{
    database::DatabaseRow,
    state::{AppState, DatabaseStore, DatabaseStoreEvent},
    ui::{database::database_sql_editor::DatabaseSqlEditor, translated::ts},
};

pub struct DatabaseSqlExecutor {
    /// Currently loaded set of results
    rows: Vec<DatabaseRow>,

    /// Background task for loading results
    rows_task: Option<Task<()>>,

    /// State for the results table
    table_state: Entity<TableState<ResultsTableDelegate>>,

    // SQL Editor state
    editor: Entity<DatabaseSqlEditor>,
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

impl DatabaseSqlExecutor {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let table_delegate = ResultsTableDelegate::new();
        let table_state = cx.new(|cx| TableState::new(table_delegate, window, cx));

        let editor = DatabaseSqlEditor::new(window, cx, "".into(), false);

        cx.new(|cx| {
            let app = cx.global::<AppState>();
            let database_store = app.database_store.clone();

            cx.subscribe_in(
                &database_store,
                window,
                |this: &mut DatabaseSqlExecutor, _database_store, event, _window, cx| match event {
                    DatabaseStoreEvent::DatabaseChanged => {
                        this.update_result_rows(vec![], cx);
                    }
                },
            )
            .detach();

            Self {
                rows: Vec::new(),
                table_state,
                rows_task: None,
                editor,
            }
        })
    }

    fn update_result_rows(&mut self, rows: Vec<DatabaseRow>, cx: &mut gpui::Context<'_, Self>) {
        self.rows = rows.clone();

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

    fn perform_query(
        &mut self,
        database_store: &Entity<DatabaseStore>,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<'_, Self>,
        query: SharedString,
    ) {
        // Drop the current task to abort it
        _ = self.rows_task.take();

        let database_store = database_store.read(cx);
        let database = match database_store.database.as_ref() {
            Some(value) => value.clone(),
            None => return,
        };

        let task = cx.spawn_in(window, async move |this, cx| {
            tracing::debug!("performing database tables load");

            let rows = match database.query(query.as_ref()).await {
                Ok(value) => value,
                Err(error) => {
                    tracing::error!(?error, "failed to query database tables");

                    // TODO: Display error
                    _ = this.update(cx, |this, cx| {
                        this.rows_task = None;
                        this.update_result_rows(Vec::new(), cx);
                    });

                    return;
                }
            };

            _ = this.update(cx, |this, cx| {
                this.rows_task = None;

                tracing::debug!(?rows, "loaded database tables");
                this.update_result_rows(rows, cx);
            });
        });

        self.rows_task = Some(task);
    }
}

impl Render for DatabaseSqlExecutor {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
            .v_flex()
            .size_full()
            .child(
                Button::new("execute")
                    .child(ts("execute"))
                    .small()
                    .on_click(cx.listener(|this, _event, window, cx| {
                        let app_state = cx.global::<AppState>();
                        let database_store = app_state.database_store.clone();
                        let editor = this.editor.read(cx);
                        let query = editor.input_state.read(cx).value();

                        this.perform_query(&database_store, window, cx, query);
                    })),
            )
            .child(self.editor.clone())
            .child(if self.rows_task.is_some() {
                div()
                    .size_full()
                    .justify_center()
                    //
                    .child(Spinner::new())
            } else {
                div()
                    .size_full()
                    //
                    .child(
                        Table::new(&self.table_state)
                            .stripe(true)
                            .bordered(true)
                            .scrollbar_visible(true, true),
                    )
            })
    }
}
