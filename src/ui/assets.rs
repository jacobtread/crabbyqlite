use anyhow::anyhow;
use gpui::{AssetSource, Result, SharedString};
use gpui_component_assets::Assets;
use rust_embed::RustEmbed;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "icons/**/*.svg"]
pub struct CustomAssets;

impl AssetSource for CustomAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        Self::get(path)
            .map(|f| Some(f.data))
            .ok_or_else(|| anyhow!("could not find asset at path \"{path}\""))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| p.starts_with(path).then(|| p.into()))
            .collect())
    }
}

/// Custom asset source that uses a combination of the gpui-component assets
/// and the local custom assets
pub struct CombinedAssetSource {
    pub assets: Assets,
    pub custom_assets: CustomAssets,
}

impl AssetSource for CombinedAssetSource {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Ok(Some(value)) = self.assets.load(path) {
            return Ok(Some(value));
        }

        self.custom_assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut first_list = self.assets.list(path)?;
        let second_list = self.custom_assets.list(path)?;

        first_list.extend_from_slice(&second_list);
        Ok(first_list)
    }
}
