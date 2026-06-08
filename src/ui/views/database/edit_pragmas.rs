use std::{collections::HashMap, rc::Rc};

use gpui::{
    App, AppContext, AsyncWindowContext, Context, Div, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
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
    database::{
        AnySharedDatabase, PragmaDefinition, PragmaType, sqlite::pragma::PRAGMA_DEFINITIONS,
    },
    state::{async_resource::AsyncResource, database::DatabaseResourceExt},
    ui::components::atoms::icons::CustomIconName,
};

pub struct EditPragmasView {
    /// States for each of the pragma values, shared as the values
    /// must be accessible from a database task which updates the
    /// current values
    states: Rc<Mutex<HashMap<&'static str, PragmaState>>>,
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
            Self::load_pragma_state(definition, states.as_ref(), &database, cx).await;
        }
    }

    async fn load_pragma_state(
        definition: &PragmaDefinition,
        states: &Mutex<HashMap<&'static str, PragmaState>>,
        database: &AnySharedDatabase,
        cx: &mut AsyncWindowContext,
    ) {
        let sql = format!("PRAGMA {}", definition.name);
        let mut value = match database.query(&sql).await {
            Ok(value) => value,
            Err(error) => {
                tracing::error!(
                    ?error,
                    name = definition.name,
                    "unable to retrieve pragma value"
                );
                return;
            }
        };

        let value = match value.rows.pop().and_then(|mut row| row.values.pop()) {
            Some(value) => value,
            None => {
                return;
            }
        };

        tracing::debug!("PRAGMA {} = {}", definition.name, value);
        Self::set_pragma_state(definition, &states, value, cx);
    }

    fn set_pragma_state(
        definition: &PragmaDefinition,
        states: &Mutex<HashMap<&'static str, PragmaState>>,
        value: SharedString,
        cx: &mut AsyncWindowContext,
    ) {
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
            _ => {}
        }
    }

    async fn set_pragma_value(
        database: AnySharedDatabase,
        definition: &PragmaDefinition,
        value: &str,
        states: Rc<Mutex<HashMap<&'static str, PragmaState>>>,
        cx: &mut AsyncWindowContext,
    ) {
        let sql = format!("PRAGMA {} = {}", definition.name, value);
        let _value = match database.query(&sql).await {
            Ok(value) => value,
            Err(error) => {
                tracing::error!(
                    ?error,
                    name = definition.name,
                    %value,
                    "unable to set pragma value"
                );
                return;
            }
        };

        Self::load_pragma_state(definition, states.as_ref(), &database, cx).await;
    }

    fn update_boolean_pragma(
        &mut self,
        definition: &'static PragmaDefinition,
        value: bool,
        window: &mut Window,
        cx: &mut Context<EditPragmasView>,
    ) {
        let database = match cx.database_connection() {
            Some(value) => value,
            _ => return,
        };

        let states = self.states.clone();

        cx.spawn_in(window, async move |_this, cx| {
            tracing::debug!("updating boolean value");
            Self::set_pragma_value(
                database,
                definition,
                match value {
                    true => "1",
                    false => "0",
                },
                states,
                cx,
            )
            .await;
        })
        .detach();
    }
}

fn boolean_pragma_value(
    definition: &'static PragmaDefinition,
    state: &PragmaStateBoolean,
    cx: &mut Context<EditPragmasView>,
) -> Div {
    div().child(
        Switch::new(definition.name)
            .checked(state.value)
            .on_click(cx.listener(move |this, checked, window, cx| {
                this.update_boolean_pragma(definition, *checked, window, cx);
            })),
    )
}

fn enum_pragma_value(state: &PragmaStateEnum) -> Div {
    div()
        .h_flex()
        .justify_end()
        .flex_auto()
        .child(Select::new(&state.state).placeholder("Select a value..."))
}

fn integer_pragma_value(
    definition: &'static PragmaDefinition,
    state: &PragmaStateInteger,
    cx: &mut Context<EditPragmasView>,
) -> Div {
    div()
        .h_flex()
        .justify_end()
        .flex_auto()
        .gap_1()
        .child(Input::new(&state.state).max_w_40().flex_auto())
        .child(
            Button::new(definition.name)
                .icon(CustomIconName::Save)
                .on_click(cx.listener(move |this, event, window, cx| {})),
        )
}

fn text_pragma_value(
    definition: &'static PragmaDefinition,
    state: &PragmaStateText,
    cx: &mut Context<EditPragmasView>,
) -> Div {
    div()
        .h_flex()
        .justify_end()
        .flex_auto()
        .gap_1()
        .child(Input::new(&state.state).max_w_40().flex_auto())
        .child(
            Button::new(definition.name)
                .icon(CustomIconName::Save)
                .on_click(cx.listener(move |this, event, window, cx| {})),
        )
}

fn definition_details(
    definition: &'static PragmaDefinition,
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
                                        PragmaState::Integer(state) => {
                                            integer_pragma_value(definition, state, cx)
                                        }
                                        PragmaState::Text(state) => {
                                            text_pragma_value(definition, state, cx)
                                        }
                                    }),
                            )
                        })),
                ),
        )
    }
}
