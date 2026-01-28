use gpui::{
    App, AppContext, Context, Element, ElementId, Entity, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Task, Window, div, px,
};
use gpui_component::{
    spinner::Spinner,
    table::{Column, Table, TableDelegate, TableState},
    tooltip::Tooltip,
};

use crate::{
    database::DatabaseTable,
    state::{AppState, DatabaseStore, DatabaseStoreEvent},
    ui::database::database_sql_editor::DatabaseSqlEditor,
};

pub struct DatabaseTablesView {
    /// Currently loaded set of database tables
    tables: Vec<DatabaseTable>,

    /// Background task for loading tables
    tables_task: Option<Task<()>>,

    /// State for the database table
    table_state: Entity<TableState<DatabaseTableDelegate>>,
}

struct DatabaseTableRow {
    name: SharedString,
    sql: SharedString,
}

struct DatabaseTableDelegate {
    data: Vec<DatabaseTableRow>,
    columns: Vec<Column>,
}

impl DatabaseTableDelegate {
    fn new() -> Self {
        Self {
            data: vec![],
            columns: vec![
                Column::new("name", "Name").width(150.).sortable(),
                Column::new("schema", "Schema").width(400.),
            ],
        }
    }
}

impl TableDelegate for DatabaseTableDelegate {
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
        let col = &self.columns[col_ix];

        let sql = row.sql.clone();

        match col.key.as_ref() {
            "name" => row.name.clone().into_any_element(),
            "schema" => div()
                .child(sql.clone())
                .id(ElementId::Name(
                    format!("schema-tooltip-{row_ix}-{col_ix}").into(),
                ))
                .tooltip(move |window, cx| {
                    let sql = sql.clone();

                    Tooltip::element(move |window, cx| {
                        let editor = DatabaseSqlEditor::new(window, cx, sql.clone(), true);

                        div()
                            //
                            .w(px(400.0))
                            .h(px(400.0))
                            .child(editor)
                            .overflow_hidden()
                    })
                    .build(window, cx)
                })
                .into_any(),

            _ => div().into_any(),
        }
    }
}

impl DatabaseTablesView {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let table_delegate = DatabaseTableDelegate::new();
        let table_state = cx.new(|cx| TableState::new(table_delegate, window, cx));

        cx.new(|cx| {
            let app = cx.global::<AppState>();
            let database_store = app.database_store.clone();

            cx.subscribe_in(
                &database_store,
                window,
                |this: &mut DatabaseTablesView, database_store, event, window, cx| match event {
                    DatabaseStoreEvent::DatabaseChanged => {
                        this.load_database_tables(database_store, window, cx);
                    }
                },
            )
            .detach();

            Self {
                tables: Vec::new(),
                table_state,
                tables_task: None,
            }
        })
    }

    fn update_database_tables(
        &mut self,
        tables: Vec<DatabaseTable>,
        cx: &mut gpui::Context<'_, Self>,
    ) {
        self.tables = tables.clone();

        self.table_state.update(cx, |this, cx| {
            this.delegate_mut().data = tables
                .into_iter()
                .map(|table| DatabaseTableRow {
                    name: table.name.into(),
                    sql: table.sql.into(),
                })
                .collect();
            this.refresh(cx);
        });
    }

    fn load_database_tables(
        &mut self,
        database_store: &Entity<DatabaseStore>,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<'_, Self>,
    ) {
        // Drop the current task to abort it
        _ = self.tables_task.take();

        // Clear the current data
        self.update_database_tables(Vec::new(), cx);

        let database_store = database_store.read(cx);
        let database = match database_store.database.as_ref() {
            Some(value) => value.clone(),
            None => return,
        };

        let task = cx.spawn_in(window, async move |this, cx| {
            tracing::debug!("performing database tables load");

            let tables = match database.database_tables().await {
                Ok(value) => value,
                Err(error) => {
                    tracing::error!(?error, "failed to query database tables");

                    // TODO: Display error
                    _ = this.update(cx, |this, cx| {
                        this.tables_task = None;
                        this.update_database_tables(Vec::new(), cx);
                    });

                    return;
                }
            };

            _ = this.update(cx, |this, cx| {
                this.tables_task = None;

                tracing::debug!(?tables, "loaded database tables");
                this.update_database_tables(tables, cx);
            });
        });

        self.tables_task = Some(task);
    }
}

impl Render for DatabaseTablesView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        if self.tables_task.is_some() {
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
        }
    }
}
