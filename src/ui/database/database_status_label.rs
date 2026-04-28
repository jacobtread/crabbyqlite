use gpui::{
    AnyView, App, AppContext, Context, Entity, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window, div, px, rems,
};
use gpui_component::{
    ActiveTheme, IconName, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    description_list::DescriptionList,
    tooltip::Tooltip,
};

use crate::{
    database::AnySharedDatabase,
    state::{AppStateExt, async_resource::AsyncResource},
    ui::{actions::close_database::CloseDatabase, translated::ts},
};

pub struct DatabaseStatusLabel {
    database_options: Option<SharedDatabaseOptions>,
}

#[derive(Debug, Clone)]
pub struct SharedDatabaseOptions {
    /// Path to the database file
    pub path: SharedString,

    /// Whether the db is readonly
    pub readonly: bool,

    /// Whether the db is encrypted
    pub encrypted: bool,
}

impl DatabaseStatusLabel {
    pub fn new(_window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let database = cx.database();

            cx.observe(&database, |this: &mut DatabaseStatusLabel, database, cx| {
                this.update_database_options(&database, cx);
            })
            .detach();

            DatabaseStatusLabel {
                database_options: None,
            }
        })
    }

    fn update_database_options(
        &mut self,
        database_store: &Entity<AsyncResource<AnySharedDatabase>>,
        cx: &mut Context<'_, Self>,
    ) {
        let database = match database_store.read(cx) {
            AsyncResource::Loaded(value) => value,
            _ => {
                self.database_options = None;
                return;
            }
        };

        let options = database.options();
        self.database_options = Some(SharedDatabaseOptions {
            path: options.path.into(),
            encrypted: options.encrypted,
            readonly: options.readonly,
        });
    }
}

impl Render for DatabaseStatusLabel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.database_options.clone() {
            Some(options) => div()
                .id("database-label")
                .bg(cx.theme().success)
                .items_center()
                .p(px(2.0))
                .px(px(4.0))
                .h_flex()
                .gap_1()
                .rounded_sm()
                .text_color(cx.theme().button_primary_foreground)
                .text_size(rems(0.65))
                .tooltip({
                    let options = options.clone();
                    move |window, cx| DatabaseOptionsTooltip::new(options.clone()).build(window, cx)
                })
                .child(options.path)
                .child(
                    Button::new("close-database")
                        .success()
                        .icon(IconName::Close)
                        .text()
                        .cursor_pointer()
                        .xsmall()
                        .text_color(cx.theme().button_primary_foreground)
                        .on_mouse_down(MouseButton::Left, |_event, window, cx| {
                            cx.stop_propagation();
                            window.dispatch_action(Box::new(CloseDatabase), cx);
                        }),
                ),

            None => div()
                .id("database-label")
                .bg(cx.theme().warning)
                .p(px(2.0))
                .px(px(4.0))
                .rounded_sm()
                .text_color(cx.theme().button_primary_foreground)
                .text_size(rems(0.65))
                .child(ts("not-connected")),
        }
    }
}

pub struct DatabaseOptionsTooltip {
    options: SharedDatabaseOptions,
}

impl DatabaseOptionsTooltip {
    pub fn new(options: SharedDatabaseOptions) -> Self {
        Self { options }
    }

    pub fn build(self, window: &mut Window, cx: &mut App) -> AnyView {
        Tooltip::element(move |_window, _cx| {
            let options = self.options.clone();

            div().py_2().v_flex().w_80().child(
                DescriptionList::horizontal()
                    .columns(2)
                    .item("Path", options.path, 2)
                    .item("Read Only", options.readonly.to_string(), 2)
                    .item("Encrypted", options.encrypted.to_string(), 2),
            )
        })
        //
        .build(window, cx)
    }
}
