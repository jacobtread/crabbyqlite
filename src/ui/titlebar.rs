use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Window, div,
};
use gpui_component::{
    IconName, Sizable, StyledExt, TitleBar,
    button::{Button, ButtonVariants},
    label::Label,
    menu::DropdownMenu,
    tag::Tag,
    tooltip::Tooltip,
};

use crate::{
    state::{AppState, DatabaseStore, DatabaseStoreEvent},
    ui::actions::{
        close_database::CloseDatabase, new_database::NewDatabase,
        new_memory_database::NewMemoryDatabase, open_file::OpenFile,
    },
};

#[derive(Clone)]
struct DatabaseName {
    primary: SharedString,
    secondary: SharedString,
}

pub struct AppTitleBar {
    database_name: Option<DatabaseName>,
}

impl AppTitleBar {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let app = cx.global::<AppState>();
            let database_store = app.database_store.clone();

            cx.subscribe_in(
                &database_store,
                window,
                |this: &mut AppTitleBar, database_store, event, _window, cx| match event {
                    DatabaseStoreEvent::DatabaseChanged => {
                        this.set_database_name(database_store, cx);
                    }
                },
            )
            .detach();

            AppTitleBar {
                database_name: None,
            }
        })
    }

    fn set_database_name(
        &mut self,
        database_store: &Entity<DatabaseStore>,
        cx: &mut Context<'_, Self>,
    ) {
        let database_store = database_store.read(cx);
        self.database_name = database_store.database.as_ref().map(|db| {
            let name = db.name();
            DatabaseName {
                primary: name.primary.into(),
                secondary: name.secondary.into(),
            }
        });
    }
}

impl Render for AppTitleBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        TitleBar::new()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(Button::new("menu").icon(IconName::Menu).small())
                    .child(Label::new("Crabbyqlite").text_xs())
                    .child(Button::new("file").label("File").xsmall().dropdown_menu(
                        |menu, _, _| {
                            menu.menu("New Database", Box::new(NewDatabase))
                                .menu("New In-Memory Database", Box::new(NewMemoryDatabase))
                                .separator()
                                .menu("Open Database", Box::new(OpenFile))
                                .menu("Open Read-Only Database", Box::new(OpenFile))
                        },
                    ))
                    .child(Button::new("edit").label("Edit").xsmall().dropdown_menu(
                        |menu, _, _| {
                            menu.menu("Create Table", Box::new(NewDatabase))
                                .menu("Modify Table", Box::new(NewMemoryDatabase))
                                .menu("Delete Table", Box::new(NewMemoryDatabase))
                                .separator()
                                .menu("Create Index", Box::new(OpenFile))
                        },
                    ))
                    .child(Button::new("tools").label("Tools").xsmall().dropdown_menu(
                        |menu, _, _| {
                            menu.menu("Compact Database", Box::new(NewDatabase))
                                .separator()
                                .menu("Load Extension", Box::new(NewMemoryDatabase))
                        },
                    ))
                    .child(match self.database_name.clone() {
                        Some(name) => Tag::success()
                            .small()
                            .pr_1()
                            .child(
                                div()
                                    .items_center()
                                    .h_flex()
                                    .gap_1()
                                    .id("database-name")
                                    .child(name.primary)
                                    .tooltip(move |window, cx| {
                                        Tooltip::new(name.secondary.clone()).build(window, cx)
                                    })
                                    .child(
                                        Button::new("close-database")
                                            .success()
                                            .icon(IconName::Close)
                                            .on_click(|_event, window, cx| {
                                                window.dispatch_action(Box::new(CloseDatabase), cx);
                                            })
                                            .xsmall(),
                                    ),
                            )
                            .text_xs(),
                        None => Tag::warning().small().child("Not Connected").text_xs(),
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(Button::new("settings").icon(IconName::Settings)),
            )
    }
}
