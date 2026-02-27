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
    database::AnySharedDatabase,
    state::{AppStateExt, async_resource::AsyncResource},
    ui::{
        actions::{
            close_database::CloseDatabase, new_database::NewDatabase,
            new_memory_database::NewMemoryDatabase, open_file::OpenFile,
        },
        translated::ts,
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
    pub fn new(_window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let database = cx.database();

            cx.observe(&database, |this: &mut AppTitleBar, database, cx| {
                this.set_database_name(&database, cx);
            })
            .detach();

            AppTitleBar {
                database_name: None,
            }
        })
    }

    fn set_database_name(
        &mut self,
        database_store: &Entity<AsyncResource<AnySharedDatabase>>,
        cx: &mut Context<'_, Self>,
    ) {
        let database = match database_store.read(cx) {
            AsyncResource::Loaded(value) => value,
            _ => {
                self.database_name = None;
                return;
            }
        };

        let name = database.name();
        self.database_name = Some(DatabaseName {
            primary: name.primary.into(),
            secondary: name.secondary.into(),
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
                    .child(
                        Button::new("file")
                            .label(ts("file"))
                            .xsmall()
                            .dropdown_menu(|menu, _, _| {
                                menu.menu(ts("new-database"), Box::new(NewDatabase))
                                    .menu(ts("new-in-memory-database"), Box::new(NewMemoryDatabase))
                                    .separator()
                                    .menu(ts("open-database"), Box::new(OpenFile))
                                    .menu(ts("open-read-only-database"), Box::new(OpenFile))
                            }),
                    )
                    .child(
                        Button::new("edit")
                            .label(ts("edit"))
                            .xsmall()
                            .dropdown_menu(|menu, _, _| {
                                menu.menu(ts("create-table"), Box::new(NewDatabase))
                                    .menu(ts("modify-table"), Box::new(NewMemoryDatabase))
                                    .menu(ts("delete-table"), Box::new(NewMemoryDatabase))
                                    .separator()
                                    .menu(ts("create-index"), Box::new(OpenFile))
                            }),
                    )
                    .child(
                        Button::new("tools")
                            .label(ts("tools"))
                            .xsmall()
                            .dropdown_menu(|menu, _, _| {
                                menu.menu(ts("compact-database"), Box::new(NewDatabase))
                                    .separator()
                                    .menu(ts("load-extension"), Box::new(NewMemoryDatabase))
                            }),
                    )
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
                        None => Tag::warning().small().child(ts("not-connected")).text_xs(),
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
