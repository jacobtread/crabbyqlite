use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};
use gpui_component::{
    IconName, Sizable, TitleBar, button::Button, label::Label, menu::DropdownMenu,
};

use crate::ui::actions::{
    new_database::NewDatabase, new_memory_database::NewMemoryDatabase, open_file::OpenFile,
};

pub struct AppTitleBar;

impl AppTitleBar {
    pub fn new(_window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|_cx| AppTitleBar)
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
                    )),
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
