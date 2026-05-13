#[derive(Debug, Clone)]
pub struct AssetPickerRequest {
    pub asset_kind: String,
    pub current_asset: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AssetPickerOption {
    pub id: String,
    pub label: String,
}

