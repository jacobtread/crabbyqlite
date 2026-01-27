use gpui::{App, AppContext, Entity, Render, SharedString, Styled, Window};
use gpui_component::input::{Input, InputState};

pub struct DatabaseSqlEditor {
    immutable: bool,
    pub input_state: Entity<InputState>,
}

impl DatabaseSqlEditor {
    pub fn new(
        window: &mut Window,
        cx: &mut App,
        default_value: SharedString,
        immutable: bool,
    ) -> Entity<Self> {
        cx.new(|cx| DatabaseSqlEditor {
            input_state: cx.new(|cx| {
                InputState::new(window, cx)
                    .code_editor("sql")
                    .multi_line(true)
                    .soft_wrap(true)
                    .rows(6)
                    .default_value(default_value)
            }),
            immutable,
        })
    }
}

impl Render for DatabaseSqlEditor {
    fn render(
        &mut self,
        _window: &mut Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        Input::new(&self.input_state)
            .size_full()
            .disabled(self.immutable)
    }
}
