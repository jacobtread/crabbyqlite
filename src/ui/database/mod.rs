use crate::ui::database::database_tables_view::DatabaseTablesView;
use gpui::{App, AppContext, Element, Entity, ParentElement, Render, Styled, Window, div};
use gpui_component::{
    StyledExt,
    tab::{Tab, TabBar},
};

mod database_tables_view;

pub struct DatabaseView {
    active_tab: usize,
    tables_view: Entity<DatabaseTablesView>,
}

impl DatabaseView {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| DatabaseView {
            active_tab: 0,
            tables_view: DatabaseTablesView::new(window, cx),
        })
    }
}

impl Render for DatabaseView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
            .size_full()
            .v_flex()
            .child(
                TabBar::new("tabs")
                    .selected_index(self.active_tab)
                    .on_click(cx.listener(|view, index, _, cx| {
                        view.active_tab = *index;
                        cx.notify();
                    }))
                    .child(Tab::new().label("Database Structure"))
                    .child(Tab::new().label("Browse Data"))
                    .child(Tab::new().label("Edit Pragmas"))
                    .child(Tab::new().label("Execute SQL")),
            )
            .child(match self.active_tab {
                0 => div().size_full().child(self.tables_view.clone()).into_any(),
                1 => div().child("Browse Data").into_any(),
                2 => div().child("Edit Pragmas").into_any(),
                3 => div().child("Execute SQL").into_any(),
                _ => div().into_any(),
            })
    }
}
