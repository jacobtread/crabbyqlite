use gpui::{AnyElement, App, Entity, IntoElement, RenderOnce, SharedString, Window};
use gpui_component::{Icon, IconNamed};

#[derive(IntoElement, Clone)]
pub enum CustomIconName {
    Cable,
    Database,
    Box,
    LogoDark,
    LogoLight,
    LogoLightPng,
    Save,
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
            Self::Database => "icons/database.svg",
            Self::Box => "icons/box.svg",
            Self::LogoDark => "icons/logo-dark.svg",
            Self::LogoLight => "icons/logo-light-64x64.svg",
            Self::LogoLightPng => "icons/logo-light-64x64.png",
            Self::Save => "icons/save.svg",
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
