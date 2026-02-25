use gpui::{
    App, AppContext, Context, Element, ElementId, Entity, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};
use gpui_component::{
    spinner::Spinner,
    table::{Column, Table, TableDelegate, TableState},
    tooltip::Tooltip,
};

use crate::{
    database::DatabaseTable,
    state::{async_resource::AsyncResource, database_tables::database_tables_resource},
    ui::{database::database_sql_editor::DatabaseSqlEditor, translated::ts},
};

pub struct DatabaseTablesView {
    /// Currently loaded set of database tables
    tables: Entity<AsyncResource<Vec<DatabaseTable>>>,

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
                Column::new("name", ts("name")).width(150.).sortable(),
                Column::new("schema", ts("schema")).width(400.),
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
            let tables = database_tables_resource(window, cx);

            cx.observe(&tables, |this: &mut DatabaseTablesView, tables, cx| {
                let tables_data = match tables.read(cx) {
                    AsyncResource::Loaded(tables) => tables.clone(),
                    _ => Vec::new(),
                };

                tracing::debug!("loaded tables data for display");

                this.table_state.update(cx, |this, cx| {
                    this.delegate_mut().data = tables_data
                        .into_iter()
                        .map(|table| DatabaseTableRow {
                            name: table.name.into(),
                            sql: table.sql.into(),
                        })
                        .collect();
                    this.refresh(cx);
                });
            })
            .detach();

            Self {
                tables,
                table_state,
            }
        })
    }
}

impl Render for DatabaseTablesView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        match self.tables.read(cx) {
            //
            AsyncResource::Idle | AsyncResource::Loading(_) => div()
                .size_full()
                .justify_center()
                //
                .child(Spinner::new()),

            //
            AsyncResource::Loaded(_) => div()
                .size_full()
                //
                .child(
                    Table::new(&self.table_state)
                        .stripe(true)
                        .bordered(true)
                        .scrollbar_visible(true, true),
                ),

            // TODO: Proper error message display
            AsyncResource::Error(error) => div().child("TODO: Error message").child(error.clone()),
        }
    }
}
