use crate::{
    state::{async_resource::AsyncResource, database::DatabaseResourceExt},
    ui::{
        components::organisms::titlebar::AppTitleBar,
        views::{database::DatabaseView, welcome::WelcomeView},
    },
};
use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
};
use gpui_component::{Root, StyledExt};

pub struct MainApp {
    app_title_bar: Entity<AppTitleBar>,
    database_view: Entity<DatabaseView>,
}

impl MainApp {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<MainApp> {
        cx.new(|cx| MainApp {
            app_title_bar: AppTitleBar::new(window, cx),
            database_view: DatabaseView::new(window, cx),
        })
    }
}

impl Render for MainApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);
        let database = cx.database();

        div()
            .v_flex()
            .size_full()
            .child(self.app_title_bar.clone())
            .child(div().size_full().child(match database.read(cx) {
                AsyncResource::Idle => WelcomeView.into_any_element(),
                _ => self.database_view.clone().into_any_element(),
            }))
            .children(notification_layer)
            .children(dialog_layer)
    }
}
