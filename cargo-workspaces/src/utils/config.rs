use crate::utils::{Error, Result};

use serde::Deserialize;
use serde_json::{from_value, Value};

#[derive(Deserialize, Default)]
struct MetadataWorkspaces<T> {
    pub workspaces: Option<T>,
}

// TODO: Validation of conflicting options (hard to tell conflicts if between cli and option)
pub fn read_config<T>(value: &Value) -> Result<T>
where
    T: for<'de> Deserialize<'de> + Default,
{
    from_value::<Option<MetadataWorkspaces<T>>>(value.clone())
        .map_err(Error::BadMetadata)
        .map(|v| v.unwrap_or_default().workspaces.unwrap_or_default())
}

#[derive(Deserialize, Default, Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct PackageConfig {
    pub independent: Option<bool>,
}

#[derive(Deserialize, Default, Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct WorkspaceConfig {
    pub allow_branch: Option<String>,
    pub no_individual_tags: Option<bool>,
}
