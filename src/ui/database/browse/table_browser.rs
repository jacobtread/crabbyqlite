use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled,
    Task, Window, div,
};
use gpui_component::{
    StyledExt,
    spinner::Spinner,
    table::{Column, DataTable, TableDelegate, TableState},
};

use crate::{
    database::{AnySharedDatabase, DatabaseRow, DatabaseTableQuery},
    state::AppStateExt,
    ui::pagination::Pagination,
};

/// Component for browsing the contents of a table
pub struct DatabaseTableBrowser {
    /// The table being browsed
    table: String,
    /// Pagination of the dataset
    pagination: TablePaginationData,

    /// State for the table
    table_state: Entity<TableState<BrowseTableDelegate>>,

    /// State for the table data loading
    load_state: TableLoadState,
}

enum TableLoadState {
    Idle,
    Loading(#[allow(unused)] Task<()>),
    Loaded,
    Error(SharedString),
}

struct TablePaginationData {
    page: i64,
    page_size: i64,

    /// Total number of known items within the table, if the query
    /// has been completed and the value is known
    count: Option<i64>,
}

impl Default for TablePaginationData {
    fn default() -> Self {
        Self {
            page: 0,
            page_size: 5,
            count: None,
        }
    }
}

struct BrowseTableRow {
    values: Vec<SharedString>,
}

impl From<DatabaseRow> for BrowseTableRow {
    fn from(row: DatabaseRow) -> Self {
        BrowseTableRow {
            values: row
                .value
                .into_iter()
                .map(|column| column.value.into())
                .collect(),
        }
    }
}

#[derive(Default)]
struct BrowseTableDelegate {
    data: Vec<BrowseTableRow>,
    columns: Vec<Column>,
}

/// Helper to collect the available columns from a collection of database rows
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

impl TableDelegate for BrowseTableDelegate {
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
        _cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let row = &self.data[row_ix];
        let value = row.values.get(col_ix);
        value.cloned().unwrap_or_default()
    }
}

impl DatabaseTableBrowser {
    pub fn new(table: String, window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let table_delegate = BrowseTableDelegate::default();
            let table_state = cx.new(|cx| TableState::new(table_delegate, window, cx));
            let pagination = TablePaginationData::default();

            let mut this = Self {
                table,
                pagination,
                table_state,
                load_state: TableLoadState::Idle,
            };

            this.load_table_page(window, cx);

            this
        })
    }

    /// Set the current table rows and columns from the provided collection
    /// of rows refreshing the visible table
    fn set_rows(&mut self, rows: Vec<DatabaseRow>, cx: &mut Context<'_, Self>) {
        self.table_state.update(cx, |this, cx| {
            let delegate = this.delegate_mut();
            delegate.columns = compute_columns(&rows);
            delegate.data = rows.into_iter().map(|row| row.into()).collect();

            this.refresh(cx);
        });
    }

    /// Async loading logic to load the rows set and the total count of rows
    async fn load_table_page_async(
        database: AnySharedDatabase,
        query: DatabaseTableQuery,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<(Vec<DatabaseRow>, i64)> {
        let rows = database
            .query_table_rows(query.clone(), limit, offset)
            .await?;
        let count = database.query_table_rows_count(query.clone()).await?;
        Ok((rows, count))
    }

    /// Load the current table
    fn load_table_page(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let database = match cx.current_database() {
            Some(value) => value,
            None => {
                self.pagination.count = None;
                self.load_state = TableLoadState::Idle;
                return;
            }
        };

        let table = self.table.clone();
        let limit = self.pagination.page_size;
        let offset = self.pagination.page * limit;

        let task = cx.spawn_in(window, async move |this, cx| {
            let query = DatabaseTableQuery { table };
            let result = Self::load_table_page_async(database, query, limit, offset).await;

            _ = this.update(cx, |this, cx| match result {
                Ok((rows, count)) => {
                    this.set_rows(rows, cx);
                    this.pagination.count = Some(count);
                    this.load_state = TableLoadState::Loaded;
                }
                Err(error) => this.load_state = TableLoadState::Error(error.to_string().into()),
            });
        });

        self.load_state = TableLoadState::Loading(task);
    }
}

impl Render for DatabaseTableBrowser {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = self.pagination.page as usize;
        let page_size = self.pagination.page_size as usize;
        let total_pages = match self.pagination.count {
            Some(rows) => usize::div_ceil(rows as usize, page_size),
            None => 0,
        };

        match &self.load_state {
            TableLoadState::Idle | TableLoadState::Loading(_) => div()
                .size_full()
                .justify_center()
                //
                .child(Spinner::new()),
            TableLoadState::Loaded => div()
                .v_flex()
                .size_full()
                .child(
                    div().flex_auto().child(
                        DataTable::new(&self.table_state)
                            .stripe(true)
                            .bordered(true)
                            .scrollbar_visible(true, true),
                    ),
                )
                .child(
                    Pagination::new("browse-pagination")
                        .current_page(page + 1)
                        .total_pages(total_pages)
                        .on_click(cx.listener(|this, page, window, cx| {
                            let page: i64 = (*page as i64).saturating_sub(1);
                            if this.pagination.page == page {
                                return;
                            }

                            this.pagination.page = page;
                            this.load_table_page(window, cx);
                        })),
                ),
            TableLoadState::Error(error) => div().child("TODO: Error message").child(error.clone()),
        }
    }
}
