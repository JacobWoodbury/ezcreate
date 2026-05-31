use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionBlueprintFile {
    pub pieces: Vec<SectionBlueprintPiece>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionBlueprintPiece {
    #[serde(rename = "scenePath")]
    pub scene_path: String,
    #[serde(rename = "itemId")]
    pub item_id: String,
    pub offset: [i32; 3],
    #[serde(rename = "albedoTexturePath", skip_serializing_if = "Option::is_none")]
    pub albedo_texture_path: Option<String>,
}
