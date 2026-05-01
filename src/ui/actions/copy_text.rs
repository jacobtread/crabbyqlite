use gpui::{Action, App, AppContext, ClipboardItem, SharedString};
use gpui_component::{WindowExt, notification::Notification};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = file)]
pub struct CopyText {
    pub label: SharedString,
    pub text: SharedString,
}

pub fn copy_text(copy: &CopyText, cx: &mut App) {
    let label = copy.label.clone();
    let text = copy.text.clone();

    cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));

    let window = cx.active_window().expect("expected a active window");
    cx.defer(move |cx| {
        _ = cx.update_window(window, |_view, window, cx| {
            window.push_notification(
                Notification::new()
                    .message(label.to_string())
                    .with_type(gpui_component::notification::NotificationType::Info),
                cx,
            );
            tracing::debug!("copied text to clipboard");
        });
    });
}
