//! [DatabaseTablesTreeView] component for rendering the available databases
//! as a tree view of the tables at the top level with one nesting level
//! containing the available columns within that table

use gpui::{
    AnyView, App, AppContext, Context, ElementId, Entity, InteractiveElement, IntoElement,
    NoAction, ParentElement, Render, RenderOnce, SharedString, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder, px,
};
use gpui_component::{
    StyledExt,
    list::ListItem,
    menu::{ContextMenuExt, PopupMenu},
    tag::Tag,
    tooltip::Tooltip,
    tree::{TreeItem, TreeState, tree},
};
use sqlformat::FormatOptions;
use std::{collections::HashMap, sync::Arc};

use crate::{
    database::DatabaseTable,
    state::AppStateExt,
    ui::{
        actions::copy_text::CopyText, icons::CustomIconName, sql_editor::SqlEditor, translated::ts,
    },
};

/// Tree view of the database tables and their schema revealed by expanding
/// each tree item
pub struct DatabaseTablesTreeView {
    /// Tree state
    tree_state: Entity<TreeState>,

    /// Mapping from the tree item ID to the associated table data
    tables_data: Arc<HashMap<SharedString, TableData>>,
    /// Mapping from the tree item ID to the associated table column data
    columns_data: Arc<HashMap<SharedString, TableColumnData>>,
}

#[derive(Clone)]
struct TableData {
    name: SharedString,
    sql: SharedString,
}

#[derive(Clone)]
struct TableColumnData {
    name: SharedString,
    column_type: SharedString,
    not_null: bool,
    primary_key: bool,
}

impl DatabaseTablesTreeView {
    pub fn new(cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self {
            tables_data: Default::default(),
            columns_data: Default::default(),
            tree_state: cx.new(|cx| TreeState::new(cx)),
        })
    }

    pub fn set_entries(&mut self, items: Vec<DatabaseTable>, cx: &mut Context<Self>) {
        let mut tree_items = Vec::with_capacity(items.len());
        let mut tables_data = HashMap::new();
        let mut columns_data = HashMap::new();

        for item in items {
            let name = SharedString::from(item.name);
            let sql = SharedString::from(item.sql);
            let column_data: Vec<TableColumnData> = item
                .columns
                .into_iter()
                .map(|column| TableColumnData {
                    name: column.name.into(),
                    column_type: column.column_type.into(),
                    not_null: column.not_null,
                    primary_key: column.primary_key,
                })
                .collect();

            let tree_item = TreeItem::new(name.clone(), name.clone()).children(
                column_data
                    .iter()
                    .map(|column| TreeItem::new(column.name.clone(), column.name.clone())),
            );
            tree_items.push(tree_item);

            tables_data.insert(name.clone(), TableData { name, sql });

            for column in column_data {
                columns_data.insert(column.name.clone(), column);
            }
        }

        self.tables_data = Arc::new(tables_data);
        self.columns_data = Arc::new(columns_data);

        self.tree_state.update(cx, |tree_state, cx| {
            tree_state.set_items(tree_items, cx);
        });

        cx.notify();
    }
}

impl Render for DatabaseTablesTreeView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let tables_data = self.tables_data.clone();
        let columns_data = self.columns_data.clone();

        tree(
            &self.tree_state,
            move |ix, entry, selected, _window, _cx| {
                let item = entry.item();

                if entry.depth() == 0 {
                    let entry_data = tables_data
                        .get(&item.id)
                        .expect("table data should exist")
                        .clone();

                    return ListItem::new(ix)
                        .selected(selected)
                        .pl(px(16.) * entry.depth() + px(12.)) // Indent based on depth
                        .child(TableTreeItem::new(entry_data));
                }

                let column = columns_data
                    .get(&item.id)
                    .expect("table data should exist")
                    .clone();

                ListItem::new(ix)
                    .selected(selected)
                    .pl(px(16.) * entry.depth() + px(12.)) // Indent based on depth
                    .child(ColumnTreeItem::new(column))
            },
        )
    }
}

/// Top level table item within the tree
#[derive(IntoElement)]
struct TableTreeItem {
    /// Details about the represented table
    table_data: TableData,
}

impl TableTreeItem {
    fn new(table_data: TableData) -> Self {
        Self { table_data }
    }

    /// Helper to create the hover tooltip factory function for table tree items
    /// (Shows the formatted creation SQL)
    fn tooltip(sql: SharedString) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
        move |window, cx| {
            let sql = sql.clone();
            Tooltip::element(move |window, cx| {
                let database = cx.database();

                let options = FormatOptions::default();
                let formatted = sqlformat::format(&sql, &sqlformat::QueryParams::None, &options);

                let editor = SqlEditor::new(window, cx, formatted.into(), true, false, database);

                div()
                    //
                    .w(px(400.0))
                    .h(px(400.0))
                    .child(editor)
                    .overflow_hidden()
            })
            .build(window, cx)
        }
    }

    /// Helper to create the factory function for producing the context menu
    /// revealed when right clicking a table tree item
    fn context_menu(
        sql: SharedString,
    ) -> impl Fn(PopupMenu, &mut Window, &mut Context<PopupMenu>) -> PopupMenu + 'static {
        move |menu, _window, _cx| {
            // TODO: Browse table
            menu.menu(ts("browse-table"), Box::new(NoAction))
                .separator()
                .menu(
                    ts("copy-create-statement"),
                    Box::new(CopyText {
                        label: ts("copied-create-statement"),
                        text: sql.clone(),
                    }),
                )
        }
    }
}

impl RenderOnce for TableTreeItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl gpui::IntoElement {
        let table_name = self.table_data.name;
        let sql = self.table_data.sql;

        div()
            .h_flex()
            .gap_2()
            .child(CustomIconName::Database)
            .child(table_name.clone())
            .child(
                div()
                    .max_w_40()
                    .text_ellipsis()
                    .overflow_hidden()
                    .child(Tag::secondary().outline().child("SQL"))
                    .id(ElementId::Name(
                        format!("schema-tooltip-{table_name}").into(),
                    ))
                    .tooltip(Self::tooltip(sql.clone())),
            )
            .context_menu(Self::context_menu(sql.clone()))
    }
}

/// Nested column item within a table tree item
#[derive(IntoElement)]
struct ColumnTreeItem {
    /// Details about the represented column
    column: TableColumnData,
}

impl ColumnTreeItem {
    fn new(column: TableColumnData) -> Self {
        Self { column }
    }
}

impl RenderOnce for ColumnTreeItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let column = self.column;

        div()
            .h_flex()
            .gap_2()
            .child(CustomIconName::Box)
            .child(column.name.clone())
            .child(column.column_type.clone())
            .when(column.not_null, |this| {
                this.child(Tag::warning().outline().child("NOT NULL"))
            })
            .when(column.primary_key, |this| {
                this.child(Tag::info().outline().child("PK"))
            })
    }
}
