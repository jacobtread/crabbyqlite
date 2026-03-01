use gpui::{App, AppContext, Entity, IntoElement, Render, SharedString, Styled, Window};
use gpui_component::highlighter::{LanguageConfig, LanguageRegistry};
use gpui_component::input::{Input, InputState};

use crate::database::AnySharedDatabase;
use crate::lsp::SqlLsp;
use crate::lsp::create_sql_lsp;
use crate::state::async_resource::AsyncResource;

fn create_sql_language_config() -> LanguageConfig {
    LanguageConfig {
        name: "SQL".into(),
        language: tree_sitter_sequel::LANGUAGE.into(),
        injection_languages: vec![],
        highlights: tree_sitter_sequel::HIGHLIGHTS_QUERY.into(),
        injections: "".into(),
        locals: "".into(),
    }
}

/// Initialize the sql editor (Add the global SQL language)
pub fn init_sql_editor() {
    LanguageRegistry::singleton().register("sql", &create_sql_language_config());
}

/// Text editor using the SQL language
pub struct SqlEditor {
    immutable: bool,
    pub input_state: Entity<InputState>,
}

impl SqlEditor {
    pub fn new(
        window: &mut Window,
        cx: &mut App,
        default_value: SharedString,
        immutable: bool,
        database: Entity<AsyncResource<AnySharedDatabase>>,
    ) -> Entity<Self> {
        cx.new(|cx| {
            cx.observe(&database, move |this: &mut SqlEditor, database, cx| {
                let lsp = match database.read(cx) {
                    AsyncResource::Loaded(database) => {
                        let lsp = match create_sql_lsp(database.clone()) {
                            Ok(value) => value,
                            Err(error) => {
                                tracing::error!(?error, "failed to create lsp");
                                return;
                            }
                        };

                        Some(lsp)
                    }
                    _ => None,
                };

                this.input_state.update(cx, move |this, _cx| {
                    this.lsp.completion_provider = lsp.map(SqlLsp::into_completion_provider);
                });
            })
            .detach();

            SqlEditor {
                input_state: cx.new(|cx| {
                    InputState::new(window, cx)
                        .code_editor("sql")
                        .multi_line(true)
                        .soft_wrap(true)
                        .rows(6)
                        .default_value(default_value)
                }),
                immutable,
            }
        })
    }
}

impl Render for SqlEditor {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        Input::new(&self.input_state)
            .size_full()
            .disabled(self.immutable)
    }
}
