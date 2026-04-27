use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use gpui_component::{
    ActiveTheme, IconName, Sizable, StyledExt,
    button::{Button, ButtonVariants},
};

use crate::{
    database::AnySharedDatabase,
    state::{AppStateExt, async_resource::AsyncResource},
    ui::{actions::close_database::CloseDatabase, translated::ts},
};

pub struct DatabaseStatusLabel {
    database_name: Option<DatabaseName>,
}

#[derive(Clone)]
struct DatabaseName {
    primary: SharedString,
    secondary: SharedString,
}

impl DatabaseStatusLabel {
    pub fn new(_window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let database = cx.database();

            cx.observe(&database, |this: &mut DatabaseStatusLabel, database, cx| {
                this.set_database_name(&database, cx);
            })
            .detach();

            DatabaseStatusLabel {
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

impl Render for DatabaseStatusLabel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.database_name.clone() {
            Some(name) => div()
                .bg(cx.theme().success)
                .items_center()
                .p_1()
                .pr_2()
                .h_flex()
                .gap_1()
                .rounded_sm()
                .text_color(cx.theme().button_primary_foreground)
                .child(name.primary)
                .child(
                    Button::new("close-database")
                        .success()
                        .icon(IconName::Close)
                        .ghost()
                        .xsmall()
                        .on_click(|_event, window, cx| {
                            window.dispatch_action(Box::new(CloseDatabase), cx);
                        }),
                ),

            None => div()
                .bg(cx.theme().warning)
                .p_1()
                .rounded_sm()
                .text_color(cx.theme().warning_foreground)
                .text_xs()
                .child(ts("not-connected")),
        }
    }
}
