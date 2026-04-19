use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, NoAction, ParentElement,
    Render, SharedString, StatefulInteractiveElement, Styled, Window, div,
};
use gpui_component::{
    ActiveTheme, IconName, Sizable, StyledExt, TitleBar,
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
            new_memory_database::NewMemoryDatabase, open_encrypted_database::OpenFileEncrypted,
            open_file::OpenFile,
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        TitleBar::new()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(Button::new("menu").icon(IconName::Menu).small().ghost())
                    .child(Label::new("Crabbyqlite").text_xs())
                    .child(
                        Button::new("file")
                            .ghost()
                            .label(ts("file"))
                            .xsmall()
                            .dropdown_menu(|menu, _, _| {
                                menu.menu(ts("new-database"), Box::new(NewDatabase))
                                    .menu(ts("new-in-memory-database"), Box::new(NewMemoryDatabase))
                                    .separator()
                                    .menu(ts("open-database"), Box::new(OpenFile::default()))
                                    .menu(
                                        ts("open-read-only-database"),
                                        Box::new(OpenFile { read_only: true }),
                                    )
                                    .menu(
                                        ts("open-encrypted-database"),
                                        Box::new(OpenFileEncrypted { read_only: false }),
                                    )
                                    .menu(
                                        ts("open-read-only-encrypted-database"),
                                        Box::new(OpenFileEncrypted { read_only: true }),
                                    )
                            }),
                    )
                    .child(
                        Button::new("edit")
                            .ghost()
                            .label(ts("edit"))
                            .xsmall()
                            .dropdown_menu(|menu, _, _| {
                                menu.menu(ts("create-table"), Box::new(NoAction))
                                    .menu(ts("modify-table"), Box::new(NoAction))
                                    .menu(ts("delete-table"), Box::new(NoAction))
                                    .separator()
                                    .menu(ts("create-index"), Box::new(NoAction))
                            }),
                    )
                    .child(
                        Button::new("tools")
                            .ghost()
                            .label(ts("tools"))
                            .xsmall()
                            .dropdown_menu(|menu, _, _| {
                                menu.menu(ts("compact-database"), Box::new(NoAction))
                                    .separator()
                                    .menu(ts("load-extension"), Box::new(NoAction))
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
                                    .text_color(cx.theme().button_primary_foreground)
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
                                            .ghost()
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
                    .child(Button::new("settings").icon(IconName::Settings).ghost()),
            )
    }
}
