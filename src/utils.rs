use super::structs::{GOGConfig, PlatformConfig};
use rocket::async_test;
use tokio::fs;

pub async fn get_gog_remote_config(id: &String) -> Option<GOGConfig> {
    let response = reqwest::get(format!("https://remote-config.gog.com/components/galaxy_client/clients/{id}?component_version=2.0.50")).await;

    match response {
        Ok(res) => {
            let json = res.json::<GOGConfig>().await;
            if json.is_err() {
                return None;
            }

            Some(json.unwrap())
        }
        Err(_) => None,
    }
}

pub async fn read_linux_config(id: String) -> Option<PlatformConfig> {
    let data_file = fs::canonicalize(format!("./games_data/{id}.json")).await; // This will return error if path doesn't exist
    if let Ok(data_file) = data_file {
        let raw_data = fs::read_to_string(&data_file).await.unwrap();
        if let Ok(config) = serde_json::from_str::<PlatformConfig>(&raw_data) {
            return Some(config);
        };
    }
    None
}

#[async_test]
async fn check_configs() {
    let mut read_dir = fs::read_dir("./games_data/")
        .await
        .expect("Failed to read games data config");

    while let Ok(Some(file)) = read_dir.next_entry().await {
        let path = file.path();
        let raw_data = fs::read_to_string(&path).await.unwrap();
        serde_json::from_str::<PlatformConfig>(&raw_data).expect("Failed to parse json file");
    }
}
