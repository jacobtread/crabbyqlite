use gpui::{App, AppContext, Entity, ParentElement, Render, Styled, Window, div};
use gpui_component::{
    ActiveTheme, IndexPath, StyledExt, h_flex,
    select::{SearchableVec, Select, SelectState},
};

pub type TableSelectState = SelectState<SearchableVec<String>>;

/// Toolbar for the database browser options
pub struct DatabaseBrowseDataViewToolbar {
    /// State for the table selector drop down
    table_select_state: Entity<TableSelectState>,
}

impl DatabaseBrowseDataViewToolbar {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let table_select_state = cx.new(|cx| {
                SelectState::new(SearchableVec::<String>::new(Vec::new()), None, window, cx)
            });

            Self { table_select_state }
        })
    }

    pub fn table_select_state(&self) -> Entity<TableSelectState> {
        self.table_select_state.clone()
    }

    pub fn update_tables(
        &mut self,
        tables: Vec<String>,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<String> {
        self.table_select_state().update(cx, |this, cx| {
            this.set_items(SearchableVec::new(tables), window, cx);

            // Current selection is still valid
            if let Some(value) = this.selected_value() {
                return Some(value.clone());
            }

            // Select the first item if possible
            this.set_selected_index(Some(IndexPath::new(0)), window, cx);
            this.selected_value().cloned()
        })
    }
}

impl Render for DatabaseBrowseDataViewToolbar {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div().h_flex().child(
            div().w_auto().child(
                Select::new(&self.table_select_state).empty(
                    h_flex()
                        .justify_center()
                        .text_color(cx.theme().muted_foreground)
                        .child("No options available"),
                ),
            ),
        )
    }
}
