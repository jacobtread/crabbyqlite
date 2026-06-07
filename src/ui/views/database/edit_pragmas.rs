use std::{collections::HashMap, rc::Rc};

use gpui::{
    App, AppContext, AsyncWindowContext, Context, Div, Entity, IntoElement, ParentElement, Render,
    Styled, Window, div,
};
use gpui_component::{
    ActiveTheme, StyledExt,
    button::{Button, ButtonVariants},
    input::{Input, InputState},
    scroll::ScrollableElement,
    select::{SearchableVec, Select, SelectState},
    switch::Switch,
};
use parking_lot::Mutex;

use crate::{
    database::AnySharedDatabase,
    state::{async_resource::AsyncResource, database::DatabaseResourceExt},
};

pub struct EditPragmasView {
    /// States for each of the pragma values, shared as the values
    /// must be accessible from a database task which updates the
    /// current values
    states: Rc<Mutex<HashMap<&'static str, PragmaState>>>,
}

pub struct PragmaDefinition {
    name: &'static str,
    url: &'static str,
    ty: PragmaType,
}

const fn p_definition(name: &'static str, url: &'static str, ty: PragmaType) -> PragmaDefinition {
    PragmaDefinition { name, url, ty }
}

const PRAGMA_DEFINITIONS: &[PragmaDefinition] = &[
    p_definition(
        "analysis_limit",
        "https://www.sqlite.org/pragma.html#pragma_analysis_limit",
        PragmaType::Integer,
    ),
    p_definition(
        "application_id",
        "https://www.sqlite.org/pragma.html#application_id",
        PragmaType::Integer,
    ),
    p_definition(
        "auto_vacuum",
        "https://www.sqlite.org/pragma.html#auto_vacuum",
        PragmaType::Enum {
            values: &["0", "NONE", "1", "FULL", "2", "INCREMENTAL"],
        },
    ),
    p_definition(
        "automatic_index",
        "https://www.sqlite.org/pragma.html#automatic_index",
        PragmaType::Boolean,
    ),
    p_definition(
        "cache_size",
        "https://www.sqlite.org/pragma.html#cache_size",
        PragmaType::Integer,
    ),
    p_definition(
        "cache_spill",
        "https://www.sqlite.org/pragma.html#cache_spill",
        PragmaType::Boolean,
    ),
    p_definition(
        "case_sensitive_like",
        "https://www.sqlite.org/pragma.html#case_sensitive_like",
        PragmaType::Boolean,
    ),
    p_definition(
        "cell_size_check",
        "https://www.sqlite.org/pragma.html#cell_size_check",
        PragmaType::Boolean,
    ),
    p_definition(
        "checkpoint_fullfsync",
        "https://www.sqlite.org/pragma.html#checkpoint_fullfsync",
        PragmaType::Boolean,
    ),
    p_definition(
        "count_changes",
        "https://www.sqlite.org/pragma.html#count_changes",
        PragmaType::Boolean,
    ),
    p_definition(
        "data_store_directory",
        "https://www.sqlite.org/pragma.html#data_store_directory",
        PragmaType::Text,
    ),
    p_definition(
        "default_cache_size",
        "https://www.sqlite.org/pragma.html#default_cache_size",
        PragmaType::Integer,
    ),
    p_definition(
        "defer_foreign_keys",
        "https://www.sqlite.org/pragma.html#defer_foreign_keys",
        PragmaType::Boolean,
    ),
    p_definition(
        "empty_result_callbacks",
        "https://www.sqlite.org/pragma.html#empty_result_callbacks",
        PragmaType::Boolean,
    ),
    p_definition(
        "encoding",
        "https://www.sqlite.org/pragma.html#encoding",
        PragmaType::Enum {
            values: &["UTF-8", "UTF-16", "UTF-16le", "UTF-16be"],
        },
    ),
    p_definition(
        "foreign_keys",
        "https://www.sqlite.org/pragma.html#foreign_keys",
        PragmaType::Boolean,
    ),
    p_definition(
        "full_column_names",
        "https://www.sqlite.org/pragma.html#full_column_names",
        PragmaType::Boolean,
    ),
    p_definition(
        "fullfsync",
        "https://www.sqlite.org/pragma.html#fullfsync",
        PragmaType::Boolean,
    ),
    p_definition(
        "hard_heap_limit",
        "https://www.sqlite.org/pragma.html#hard_heap_limit",
        PragmaType::Integer,
    ),
    p_definition(
        "ignore_check_constraints",
        "https://www.sqlite.org/pragma.html#ignore_check_constraints",
        PragmaType::Boolean,
    ),
    p_definition(
        "journal_mode",
        "https://www.sqlite.org/pragma.html#journal_mode",
        PragmaType::Enum {
            values: &["DELETE", "TRUNCATE", "PERSIST", "MEMORY", "WAL", "OFF"],
        },
    ),
    p_definition(
        "journal_size_limit",
        "https://www.sqlite.org/pragma.html#journal_size_limit",
        PragmaType::Integer,
    ),
    p_definition(
        "legacy_alter_table",
        "https://www.sqlite.org/pragma.html#legacy_alter_table",
        PragmaType::Boolean,
    ),
    p_definition(
        "locking_mode",
        "https://www.sqlite.org/pragma.html#locking_mode",
        PragmaType::Enum {
            values: &["NORMAL", "EXCLUSIVE"],
        },
    ),
    p_definition(
        "max_page_count",
        "https://www.sqlite.org/pragma.html#max_page_count",
        PragmaType::Integer,
    ),
    p_definition(
        "mmap_size",
        "https://www.sqlite.org/pragma.html#mmap_size",
        PragmaType::Integer,
    ),
    p_definition(
        "page_size",
        "https://www.sqlite.org/pragma.html#page_size",
        PragmaType::Integer,
    ),
    p_definition(
        "parser_trace",
        "https://www.sqlite.org/pragma.html#parser_trace",
        PragmaType::Boolean,
    ),
    p_definition(
        "query_only",
        "https://www.sqlite.org/pragma.html#query_only",
        PragmaType::Boolean,
    ),
    p_definition(
        "read_uncommitted",
        "https://www.sqlite.org/pragma.html#read_uncommitted",
        PragmaType::Boolean,
    ),
    p_definition(
        "recursive_triggers",
        "https://www.sqlite.org/pragma.html#recursive_triggers",
        PragmaType::Boolean,
    ),
    p_definition(
        "reverse_unordered_selects",
        "https://www.sqlite.org/pragma.html#reverse_unordered_selects",
        PragmaType::Boolean,
    ),
    p_definition(
        "secure_delete",
        "https://www.sqlite.org/pragma.html#secure_delete",
        PragmaType::Enum {
            values: &["true", "false", "FAST"],
        },
    ),
    p_definition(
        "short_column_names",
        "https://www.sqlite.org/pragma.html#short_column_names",
        PragmaType::Boolean,
    ),
    p_definition(
        "soft_heap_limit",
        "https://www.sqlite.org/pragma.html#soft_heap_limit",
        PragmaType::Integer,
    ),
    p_definition(
        "synchronous",
        "https://www.sqlite.org/pragma.html#synchronous",
        PragmaType::Enum {
            values: &["0", "OFF", "1", "NORMAL", "2", "FULL", "3", "EXTRA"],
        },
    ),
    p_definition(
        "temp_store",
        "https://www.sqlite.org/pragma.html#temp_store",
        PragmaType::Enum {
            values: &["0", "DEFAULT", "1", "FILE", "2", "MEMORY"],
        },
    ),
    p_definition(
        "temp_store_directory",
        "https://www.sqlite.org/pragma.html#temp_store_directory",
        PragmaType::Text,
    ),
    p_definition(
        "threads",
        "https://www.sqlite.org/pragma.html#threads",
        PragmaType::Integer,
    ),
    p_definition(
        "trusted_schema",
        "https://www.sqlite.org/pragma.html#trusted_schema",
        PragmaType::Boolean,
    ),
    p_definition(
        "user_version",
        "https://www.sqlite.org/pragma.html#user_version",
        PragmaType::Integer,
    ),
    p_definition(
        "vdbe_addoptrace",
        "https://www.sqlite.org/pragma.html#vdbe_addoptrace",
        PragmaType::Boolean,
    ),
    p_definition(
        "vdbe_debug",
        "https://www.sqlite.org/pragma.html#vdbe_debug",
        PragmaType::Boolean,
    ),
    p_definition(
        "vdbe_listing",
        "https://www.sqlite.org/pragma.html#vdbe_listing",
        PragmaType::Boolean,
    ),
    p_definition(
        "vdbe_trace",
        "https://www.sqlite.org/pragma.html#vdbe_trace",
        PragmaType::Boolean,
    ),
    p_definition(
        "wal_autocheckpoint",
        "https://www.sqlite.org/pragma.html#wal_autocheckpoint",
        PragmaType::Integer,
    ),
    p_definition(
        "writable_schema",
        "https://www.sqlite.org/pragma.html#writable_schema",
        PragmaType::Boolean,
    ),
];

enum PragmaType {
    Enum { values: &'static [&'static str] },
    Boolean,
    Integer,
    Text,
}

struct PragmaStateEnum {
    state: Entity<SelectState<SearchableVec<String>>>,
}

struct PragmaStateBoolean {
    value: bool,
}

struct PragmaStateInteger {
    state: Entity<InputState>,
}

struct PragmaStateText {
    state: Entity<InputState>,
}

enum PragmaState {
    Enum(PragmaStateEnum),
    Boolean(PragmaStateBoolean),
    Integer(PragmaStateInteger),
    Text(PragmaStateText),
}

impl EditPragmasView {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let mut states = HashMap::new();

            for definition in PRAGMA_DEFINITIONS {
                let state = match &definition.ty {
                    PragmaType::Enum { values } => {
                        let state = cx.new(|cx| {
                            SelectState::new(
                                SearchableVec::<String>::new(
                                    values
                                        .iter()
                                        .map(|value| value.to_string())
                                        .collect::<Vec<_>>(),
                                ),
                                None,
                                window,
                                cx,
                            )
                        });

                        PragmaState::Enum(PragmaStateEnum { state })
                    }
                    PragmaType::Boolean => {
                        PragmaState::Boolean(PragmaStateBoolean { value: false })
                    }
                    PragmaType::Integer => PragmaState::Integer(PragmaStateInteger {
                        state: cx.new(|cx| InputState::new(window, cx)),
                    }),
                    PragmaType::Text => PragmaState::Text(PragmaStateText {
                        state: cx.new(|cx| InputState::new(window, cx)),
                    }),
                };

                states.insert(definition.name, state);
            }

            let states = Rc::new(Mutex::new(states));
            let database = cx.database();

            cx.observe_in(&database, window, Self::on_database_change)
                .detach();

            Self { states }
        })
    }

    /// Handles changes to the database connection
    fn on_database_change(
        &mut self,
        database: Entity<AsyncResource<AnySharedDatabase>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let database = match database.read(cx) {
            AsyncResource::Loaded(value) => value.clone(),
            _ => return,
        };

        let states = self.states.clone();

        cx.spawn_in(window, async |_this, cx| {
            Self::load_pragma_states(database, states, cx).await;
        })
        .detach();
    }

    async fn load_pragma_states(
        database: AnySharedDatabase,
        states: Rc<Mutex<HashMap<&'static str, PragmaState>>>,
        cx: &mut AsyncWindowContext,
    ) {
        for definition in PRAGMA_DEFINITIONS {
            let sql = format!("PRAGMA {}", definition.name);
            let mut value = match database.query(&sql).await {
                Ok(value) => value,
                Err(error) => {
                    tracing::error!(
                        ?error,
                        name = definition.name,
                        "unable to retrieve pragma value"
                    );
                    continue;
                }
            };

            let value = match value.rows.pop().and_then(|mut row| row.values.pop()) {
                Some(value) => value,
                None => {
                    continue;
                }
            };

            tracing::debug!("PRAGMA {} = {}", definition.name, value);

            let mut states = states.lock();
            let state = states.get_mut(definition.name).expect("state should exist");

            match (&definition.ty, state) {
                (PragmaType::Enum { .. }, PragmaState::Enum(state)) => {
                    let normalized_value = value.to_uppercase();
                    // TODO: Handle 0, 1 for true/false

                    _ = state.state.update_in(cx, move |state, window, cx| {
                        state.set_selected_value(&normalized_value, window, cx);
                    });
                }
                (PragmaType::Boolean, PragmaState::Boolean(state)) => {
                    state.value = value == "1";
                }
                (PragmaType::Integer, PragmaState::Integer(state)) => {
                    _ = state.state.update_in(cx, move |state, window, cx| {
                        state.set_value(value, window, cx);
                    });
                }
                (PragmaType::Text, PragmaState::Text(state)) => {
                    _ = state.state.update_in(cx, move |state, window, cx| {
                        state.set_value(value, window, cx);
                    });
                }
                _ => break,
            }
        }
    }
}

fn boolean_pragma_value(
    definition: &PragmaDefinition,
    state: &PragmaStateBoolean,
    cx: &mut Context<EditPragmasView>,
) -> Div {
    div().child(
        Switch::new(definition.name)
            .checked(state.value)
            .on_click(cx.listener(|_this, _checked, _window, _cx| {})),
    )
}

fn enum_pragma_value(state: &PragmaStateEnum) -> Div {
    div()
        .h_flex()
        .justify_end()
        .flex_auto()
        .child(Select::new(&state.state).placeholder("Select a value..."))
}

fn integer_pragma_value(state: &PragmaStateInteger) -> Div {
    div()
        .h_flex()
        .justify_end()
        .flex_auto()
        .child(Input::new(&state.state).max_w_40().flex_auto())
}

fn text_pragma_value(state: &PragmaStateText) -> Div {
    div()
        .h_flex()
        .justify_end()
        .flex_auto()
        .child(Input::new(&state.state).max_w_40().flex_auto())
}

fn definition_details(
    definition: &PragmaDefinition,
    cx: &mut Context<EditPragmasView>,
) -> impl IntoElement {
    div()
        .flex_auto()
        .v_flex()
        .gap_1()
        .justify_start()
        .items_start()
        .child(definition.name)
        .child(docs_link(definition.name, definition.url, cx))
}

fn docs_link(
    id: &'static str,
    link: &'static str,
    cx: &mut Context<EditPragmasView>,
) -> impl IntoElement {
    Button::new(id)
        .text_color(cx.theme().muted_foreground)
        .link()
        .label("Documentation")
        .on_click(|_event, _window, cx| {
            cx.open_url(link);
        })
        .w_auto()
}

impl Render for EditPragmasView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let states = self.states.lock();

        div().size_full().p_4().child(
            div()
                .border_1()
                .border_color(cx.theme().border)
                .size_full()
                .child(
                    div()
                        .v_flex()
                        .overflow_y_scrollbar()
                        .gap_1()
                        .p_2()
                        .children(PRAGMA_DEFINITIONS.iter().map(|definition| {
                            let state = states
                                .get(definition.name)
                                // Should always be defined
                                .expect("definition is missing state");

                            div().px_4().py_2().w_full().child(
                                div()
                                    .h_flex()
                                    .child(definition_details(definition, cx))
                                    .child(match state {
                                        PragmaState::Boolean(state) => {
                                            boolean_pragma_value(definition, state, cx)
                                        }
                                        PragmaState::Enum(state) => enum_pragma_value(state),
                                        PragmaState::Integer(state) => integer_pragma_value(state),
                                        PragmaState::Text(state) => text_pragma_value(state),
                                    }),
                            )
                        })),
                ),
        )
    }
}
