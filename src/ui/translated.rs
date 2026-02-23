use gpui::{IntoElement, SharedString};

pub struct Translated {
    key: &'static str,
}

pub fn t(key: &'static str) -> Translated {
    Translated { key }
}

impl IntoElement for Translated {
    type Element = SharedString;

    fn into_element(self) -> Self::Element {
        if let Some(translated) = crate::_rust_i18n_try_translate(&rust_i18n::locale(), &self.key) {
            SharedString::from(translated)
        } else {
            SharedString::from(self.key)
        }
    }
}
