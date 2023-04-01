use flate2::bufread::ZlibDecoder;
use gog_proxy::{structs::PlatformConfig, utils};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use sqlite::State;
use std::io::prelude::*;
use std::{cmp::min, env};
use tokio::{fs, io::AsyncWriteExt};

const GET_LINUX_NATIVE_GAMES_QUERY: &str =
    "SELECT product_id FROM products WHERE comp_systems LIKE '%l%' AND product_type='game'";

#[derive(Deserialize, Debug)]
struct GalaxyMeta {
    #[serde(rename = "clientId")]
    pub client_id: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GalaxyBuild {
    pub link: String,
    pub generation: u8,
}

#[derive(Deserialize, Debug)]
struct GalaxyBuilds {
    pub items: Vec<GalaxyBuild>,
}

async fn get_existing_clients() -> Vec<String> {
    let mut clients: Vec<String> = Vec::new();
    if let Ok(mut dir) = fs::read_dir("./games_data").await {
        if let Ok(Some(file)) = dir.next_entry().await {
            let fname = file.file_name();
            let fname = fname.to_str().unwrap();

            if let Some(id) = fname.split('.').next() {
                clients.push(String::from(id));
            }
        }
    }

    clients
}

async fn obtain_client_id(client: reqwest::Client, product: String) -> Option<String> {
    let prepared_request = client
        .get(format!(
            "https://content-system.gog.com/products/{product}/os/windows/builds?generation=2"
        ))
        .build()
        .expect("Failed to prepare request to builds");

    let response = client
        .execute(prepared_request)
        .await
        .expect("Failed to get builds");
    let data: GalaxyBuilds = response.json().await.unwrap();

    let url = if let Some(data) = data.items.get(0) {
        data.link.clone()
    } else {
        String::new()
    };

    if url.is_empty() {
        return None;
    }

    // Get meta

    let prepared_request = client
        .get(url)
        .build()
        .expect("Failed to prepare request to meta");

    let response = client
        .execute(prepared_request)
        .await
        .expect("Failed to get meta ");

    let json_data: GalaxyMeta = match data.items.get(0).unwrap().generation {
        1 => response.json().await.unwrap(),

        2 => {
            let data = response.bytes().await.expect("Failed to download meta");
            let mut decoder = ZlibDecoder::new(&data[..]);
            let mut s = String::new();
            decoder
                .read_to_string(&mut s)
                .expect("Failed to decompress meta");

            serde_json::from_str(s.as_str()).unwrap()
        }

        _ => {
            panic!("Unsupported version");
        }
    };

    json_data.client_id
}

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let defined_clients = get_existing_clients().await;
    let mut native_products: Vec<String> = Vec::new();

    let tmp_dir = env::temp_dir();
    let sqlite_path = tmp_dir.join("gogdb.sqlite");

    if fs::metadata(&sqlite_path).await.is_err() {
        // File doesn't exist, downlad it

        println!("Downloading sqlite index");
        let mut response = client
            .execute(
                client
                    .get("https://www.gogdb.org/data/index.sqlite3")
                    .build()
                    .unwrap(),
            )
            .await
            .expect("Failed to obtain sqlite index");
        let mut file = fs::File::create(&sqlite_path)
            .await
            .expect("Failed to create file sqlite file handle");

        let total_size: u64 = response
            .headers()
            .get("Content-Length")
            .unwrap()
            .to_str()
            .unwrap()
            .parse()
            .unwrap();
        let mut downloaded: u64 = 0;

        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::with_template(
                "{msg} [{elapsed_precise}] {wide_bar} {bytes}/{total_bytes}",
            )
            .unwrap(),
        );

        while let Ok(Some(mut chunk)) = response.chunk().await {
            let new_downloaded = downloaded + chunk.len() as u64;
            let new = min(new_downloaded, total_size);
            downloaded = new;
            pb.set_message("Downloading");
            pb.set_position(new);
            file.write_buf(&mut chunk)
                .await
                .expect("Failed to write chunk");
        }

        file.flush().await.unwrap();
        file.shutdown().await.unwrap();
        pb.finish_with_message("Downloaded");
    }

    let connection = sqlite::open(&sqlite_path).expect("Failed to open connection to sqlite");

    let mut query = connection
        .prepare(GET_LINUX_NATIVE_GAMES_QUERY)
        .expect("Failed to prepare query");

    while let Ok(State::Row) = query.next() {
        let id = query.read::<String, _>("product_id").unwrap();
        native_products.push(id);
    }

    let pb = ProgressBar::new(native_products.len() as u64);

    let futures = native_products
        .iter()
        .map(|product| tokio::spawn(obtain_client_id(client.clone(), product.to_string())));

    let default_platform = PlatformConfig::default();

    for future in futures {
        let joined = future.await.expect("Failed to join future");
        pb.inc(1);
        if let Some(id) = joined {
            if !defined_clients.contains(&id) {
                let remote_config = utils::get_gog_remote_config(&id).await;
                let mut f_handle = fs::File::create(format!("./games_data/{id}.json"))
                    .await
                    .expect("Failed to create a file");

                let mut platform_cfg = default_platform.clone();

                if let Some(windows_config) = remote_config {
                    platform_cfg.cloud_storage.locations =
                        windows_config.content.windows.cloud_storage.locations;
                }

                f_handle
                    .write_all(
                        serde_json::to_string_pretty(&platform_cfg)
                            .unwrap()
                            .as_bytes(),
                    )
                    .await
                    .expect("Failed to write default config");
                f_handle.shutdown().await.unwrap();
            }
        }
    }
    pb.finish();
}
