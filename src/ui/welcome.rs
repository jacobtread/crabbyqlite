use crate::ui::{
    actions::{
        new_database::NewDatabase, new_memory_database::NewMemoryDatabase,
        open_encrypted_database::OpenFileEncrypted, open_file::OpenFile,
    },
    translated::ts,
};
use gpui::{
    Action, App, Context, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_component::{
    ActiveTheme, Colorize, Icon, IconName, StyledExt,
    button::{Button, ButtonVariants},
    kbd::Kbd,
    label::Label,
    menu::{DropdownMenu, PopupMenu},
};

#[derive(IntoElement)]
pub struct WelcomeView;

impl RenderOnce for WelcomeView {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .h_full()
            .v_flex()
            .gap_2()
            .items_center()
            .justify_center()
            .child(
                div()
                    .v_flex()
                    .gap_4()
                    .w(px(400.0))
                    .child(
                        Label::new(ts("welcome-back-to-crabbyqlite"))
                            .secondary(ts("handy-sqlite-tool")),
                    )
                    .child(
                        div()
                            .v_flex()
                            .gap_2()
                            .child(
                                div()
                                    .gap_1()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .h_flex()
                                    .child(ts("get-started").to_uppercase())
                                    .child(div().flex_auto().h(px(1.0)).bg(
                                        cx.theme().secondary_foreground.lighten(0.1).opacity(0.3),
                                    )),
                            )
                            .child(
                                WelcomeButton::new(
                                    "new-database",
                                    ts("new-database"),
                                    Box::new(NewDatabase),
                                    Icon::new(IconName::Plus),
                                )
                                .dropdown_menu(|menu, _, _| {
                                    menu.menu(ts("new-database"), Box::new(NewDatabase)).menu(
                                        ts("new-in-memory-database"),
                                        Box::new(NewMemoryDatabase),
                                    )
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
                            ),
                    ),
            )
    }
}

type MenuCallback =
    Box<dyn Fn(PopupMenu, &mut Window, &mut Context<PopupMenu>) -> PopupMenu + 'static>;

#[derive(IntoElement)]
struct WelcomeButton {
    id: ElementId,
    label: SharedString,
    action: Box<dyn Action>,
    icon: Icon,
    menu: Option<MenuCallback>,
}

impl WelcomeButton {
    pub fn new(
        id: impl Into<ElementId>,
        label: SharedString,
        action: Box<dyn Action>,
        icon: Icon,
    ) -> Self {
        Self {
            id: id.into(),
            label,
            action,
            icon,
            menu: None,
        }
    }

    /// Set the dropdown menu of the button.
    pub fn dropdown_menu(
        mut self,
        menu: impl Fn(PopupMenu, &mut Window, &mut Context<PopupMenu>) -> PopupMenu + 'static,
    ) -> Self {
        self.menu = Some(Box::new(menu));
        self
    }
}

impl RenderOnce for WelcomeButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let action = self.action;

        let key_binding = Kbd::binding_for_action(action.as_ref(), None, window);

        div()
            .id(self.id)
            .child(self.icon)
            .child(self.label.clone())
            .on_click(move |_event, window, cx| {
                window.dispatch_action(action.boxed_clone(), cx);
            })
            .gap_2()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_start()
            .w_full()
            .h_8()
            // .pr_4()
            .pl_2()
            .rounded_sm()
            .cursor_pointer()
            .text_sm()
            .text_color(cx.theme().secondary_foreground)
            .bg(cx.theme().transparent)
            .hover(|style| {
                style.bg(if cx.theme().mode.is_dark() {
                    cx.theme().secondary.lighten(0.1).opacity(0.8)
                } else {
                    cx.theme().secondary.darken(0.1).opacity(0.8)
                })
            })
            // Horizontal spacer
            .child(div().flex_auto())
            .when_some(key_binding, |this, kbd| {
                this.child(
                    div()
                        .text_xs()
                        .flex_shrink_0()
                        .text_color(cx.theme().muted_foreground)
                        .child(kbd.appearance(false)),
                )
            })
            .when_some(self.menu, |this, menu| {
                this.child(
                    Button::new("popup")
                        .icon(IconName::ChevronDown)
                        .text()
                        .cursor_pointer()
                        .p_2()
                        .dropdown_menu(menu),
                )
            })
    }
}
