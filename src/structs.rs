use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveLocation {
    pub name: String,
    pub location: String,
    pub wildcard: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Overlay {
    pub supported: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CloudStorage {
    pub enabled: bool,
    pub locations: Vec<SaveLocation>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QuotaConfig {
    pub quota: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlatformConfig {
    pub overlay: Overlay,
    #[serde(rename = "cloudStorage")]
    pub cloud_storage: CloudStorage,
}

impl PlatformConfig {
    pub fn default() -> PlatformConfig {
        PlatformConfig {
            overlay: Overlay { supported: false },
            cloud_storage: CloudStorage {
                enabled: false,
                locations: Vec::new(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GOGConfigContent {
    #[serde(rename = "MacOS")]
    pub mac_os: PlatformConfig,
    #[serde(rename = "Windows")]
    pub windows: PlatformConfig,
    #[serde(rename = "Linux")]
    pub linux: Option<PlatformConfig>,
    #[serde(rename = "cloudStorage")]
    pub cloud_storage: QuotaConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GOGConfig {
    pub version: String,
    pub content: GOGConfigContent,
}
