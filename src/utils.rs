use crate::structs::GOGConfig;

pub async fn get_gog_remote_config(id: String) -> Option<GOGConfig> {
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
