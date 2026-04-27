use gpui::{
    App, AppContext, Context, Entity, IntoElement, NoAction, ParentElement, Render, Styled, Window,
    div,
};
use gpui_component::{
    IconName, Sizable, TitleBar,
    button::{Button, ButtonVariants},
    label::Label,
    menu::DropdownMenu,
};

use crate::ui::{
    actions::{
        new_database::NewDatabase, new_memory_database::NewMemoryDatabase,
        open_encrypted_database::OpenFileEncrypted, open_file::OpenFile,
    },
    database::database_status_label::DatabaseStatusLabel,
    translated::ts,
};

pub struct AppTitleBar {
    label: Entity<DatabaseStatusLabel>,
}

impl AppTitleBar {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let label = DatabaseStatusLabel::new(window, cx);

            AppTitleBar { label }
        })
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
                    .child(self.label.clone()),
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
