use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc, vec};

use gpui::{
    App, AppContext, Context, ElementId, Entity, InteractiveElement, NoAction, ParentElement,
    Render, SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
    px,
};
use gpui_component::{
    Icon, IconName, StyledExt,
    list::ListItem,
    menu::ContextMenuExt,
    tag::Tag,
    tooltip::Tooltip,
    tree::{TreeItem, TreeState, tree},
};
use sqlformat::FormatOptions;
use tracing::Instrument;

use crate::{
    database::DatabaseTable,
    state::AppStateExt,
    ui::{icons::CustomIconName, sql_editor::SqlEditor, translated::ts},
};

pub struct DatabaseTablesTreeView {
    tree_state: Entity<TreeState>,

    tables_data: Arc<HashMap<SharedString, TableData>>,
    columns_data: Arc<HashMap<SharedString, TableColumnData>>,
}

struct TableData {
    name: SharedString,
    sql: SharedString,
}

struct TableColumnData {
    name: SharedString,
    column_type: SharedString,
    not_null: bool,
    primary_key: bool,
}

impl DatabaseTablesTreeView {
    pub fn new(_window: &mut Window, cx: &mut App) -> Entity<Self> {
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
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let tables_data = self.tables_data.clone();
        let columns_data = self.columns_data.clone();

        tree(&self.tree_state, move |ix, entry, selected, window, cx| {
            let item = entry.item();

            if entry.depth() == 0 {
                let entry_data = tables_data.get(&item.id).expect("table data should exist");
                let sql = entry_data.sql.clone();

                return ListItem::new(ix)
                    .selected(selected)
                    .pl(px(16.) * entry.depth() + px(12.)) // Indent based on depth
                    .child(
                        div()
                            .h_flex()
                            .gap_2()
                            .child(CustomIconName::Database)
                            .child(item.label.clone())
                            .child(
                                div()
                                    .max_w_40()
                                    .text_ellipsis()
                                    .overflow_hidden()
                                    .child(Tag::secondary().outline().child("SQL"))
                                    .id(ElementId::Name(format!("schema-tooltip-{ix}").into()))
                                    .tooltip(move |window, cx| {
                                        let sql = sql.clone();

                                        Tooltip::element(move |window, cx| {
                                            let database = cx.database();

                                            let options = FormatOptions::default();
                                            let formatted = sqlformat::format(
                                                &sql,
                                                &sqlformat::QueryParams::None,
                                                &options,
                                            );

                                            let editor = SqlEditor::new(
                                                window,
                                                cx,
                                                formatted.into(),
                                                true,
                                                database,
                                            );

                                            div()
                                                //
                                                .w(px(400.0))
                                                .h(px(400.0))
                                                .child(editor)
                                                .overflow_hidden()
                                        })
                                        .build(window, cx)
                                    }),
                            )
                            .context_menu(|menu, window, cx| {
                                // TODO:
                                menu.menu(ts("browse-table"), Box::new(NoAction))
                                    .separator()
                                    .menu(ts("copy-create-statement"), Box::new(NoAction))
                            }),
                    );
            }
            let column = columns_data.get(&item.id).expect("table data should exist");

            ListItem::new(ix)
                .selected(selected)
                .pl(px(16.) * entry.depth() + px(12.)) // Indent based on depth
                .child(
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
                        }),
                )
        })
    }
}
