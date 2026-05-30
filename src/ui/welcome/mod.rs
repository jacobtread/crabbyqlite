use crate::ui::{
    actions::{
        new_database::NewDatabase, new_memory_database::NewMemoryDatabase,
        open_encrypted_database::OpenFileEncrypted, open_file::OpenFile,
    },
    translated::ts,
    welcome::welcome_button::WelcomeButton,
};
use gpui::{App, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window, div, img, px};
use gpui::{StyledImage, rems};
use gpui_component::{ActiveTheme, Colorize, Icon, IconName, StyledExt};

pub mod welcome_button;

#[derive(IntoElement)]
pub struct WelcomeView;

impl RenderOnce for WelcomeView {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .h_full()
            .v_flex()
            .gap_4()
            .items_center()
            .justify_center()
            .child(
                div()
                    .v_flex()
                    .gap_2()
                    .w(px(400.0))
                    .child(welcome_intro(cx))
                    .child(welcome_actions(cx)),
            )
    }
}

fn welcome_intro(cx: &mut App) -> impl IntoElement {
    div()
        .h_flex()
        .w_full()
        .gap_2()
        .child(
            img("icons/logo-light.svg")
                .object_fit(gpui::ObjectFit::ScaleDown)
                .w(px(64.))
                .h(px(64.)),
        )
        .child(
            div()
                .v_flex()
                .gap_2()
                .line_height(rems(1.25))
                .child(
                    div()
                        .child(ts("welcome-back-to-crabbyqlite"))
                        .text_color(cx.theme().foreground),
                )
                .child(
                    div()
                        .child(ts("handy-sqlite-tool"))
                        .text_color(cx.theme().muted_foreground),
                ),
        )
        .mb_4()
}

fn welcome_actions(cx: &mut App) -> impl IntoElement {
    div()
        .v_flex()
        .w_full()
        .gap_2()
        .child(
            div()
                .gap_1()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .h_flex()
                .child(ts("get-started").to_uppercase())
                .child(
                    div().flex_auto().h(px(1.0)).bg(cx
                        .theme()
                        .secondary_foreground
                        .lighten(0.1)
                        .opacity(0.3)),
                ),
        )
        .child(
            WelcomeButton::new(
                "new-database",
                ts("new-database"),
                Box::new(NewDatabase),
                Icon::new(IconName::Plus),
            )
            .dropdown_menu(|menu, _, _| {
                menu.menu(ts("new-database"), Box::new(NewDatabase))
                    .menu(ts("new-in-memory-database"), Box::new(NewMemoryDatabase))
            }),
        )
        .child(
            WelcomeButton::new(
                "open-database",
                ts("open-database"),
                Box::new(OpenFile::default()),
                Icon::new(IconName::File),
            )
            .dropdown_menu(|menu, _, _| {
                menu.menu(ts("open-database"), Box::new(OpenFile::default()))
                    .menu(
                        ts("open-read-only-database"),
                        Box::new(OpenFile { read_only: true }),
                    )
            }),
        )
        .child(
            WelcomeButton::new(
                "open-encrypted-database",
                ts("open-encrypted-database"),
                Box::new(OpenFileEncrypted { read_only: false }),
                Icon::new(IconName::File),
            )
            .dropdown_menu(|menu, _, _| {
                menu.menu(
                    ts("open-encrypted-database"),
                    Box::new(OpenFileEncrypted { read_only: false }),
                )
                .menu(
                    ts("open-read-only-encrypted-database"),
                    Box::new(OpenFileEncrypted { read_only: true }),
                )
            }),
        )
}
