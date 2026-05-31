use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ModManifest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub items: Vec<ModManifestItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModManifestItem {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "scenePath")]
    pub scene_path: String,
    #[serde(rename = "thumbnailPath", default)]
    pub thumbnail_path: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(rename = "sectionSpecPath", default)]
    pub section_spec_path: Option<String>,
}
