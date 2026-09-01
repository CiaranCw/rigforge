//! Asset Browser list rows. Not a search DSL.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetListItem {
    pub logical_id: String,
    pub display_name: String,
    pub record_type: String,
    pub published_version_id: Option<String>,
    pub draft_version_id: Option<String>,
}
