use gpui::{AnyElement, App, Entity, IntoElement, RenderOnce, SharedString, Window};
use gpui_component::{Icon, IconNamed};

#[derive(IntoElement, Clone)]
pub enum CustomIconName {
    Cable,
}

impl CustomIconName {
    /// Return the icon as a Entity<Icon>
    pub fn view(self, cx: &mut App) -> Entity<Icon> {
        Icon::from(self).view(cx)
    }
}

impl IconNamed for CustomIconName {
    fn path(self) -> SharedString {
        match self {
            Self::Cable => "icons/cable.svg",
        }
        .into()
    }
}

impl From<CustomIconName> for AnyElement {
    fn from(val: CustomIconName) -> Self {
        Icon::from(val).into_any_element()
    }
}

impl RenderOnce for CustomIconName {
    fn render(self, _: &mut Window, _cx: &mut App) -> impl IntoElement {
        Icon::from(self)
    }
}
