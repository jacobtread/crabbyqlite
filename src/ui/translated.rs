use gpui::{IntoElement, SharedString};

pub struct Translated {
    key: &'static str,
}

pub fn t(key: &'static str) -> Translated {
    Translated { key }
}

pub fn ts(key: &'static str) -> SharedString {
    if let Some(translated) = crate::_rust_i18n_try_translate(&rust_i18n::locale(), key) {
        SharedString::from(translated)
    } else {
        SharedString::from(key)
    }
}

impl IntoElement for Translated {
    type Element = SharedString;

    fn into_element(self) -> Self::Element {
        ts(self.key)
    }
}
