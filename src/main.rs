#[macro_use]
extern crate rocket;
use rocket::{http::Status, serde::json};
use structs::{GOGConfig, PlatformConfig};
use tokio::fs;

mod structs;
mod utils;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/config/<id>")]
async fn get_config(id: String) -> (Status, Option<json::Json<GOGConfig>>) {
    let gog_config = utils::get_gog_remote_config(id.clone()).await;

    if gog_config.is_none() {
        return (Status::NotFound, None);
    }

    let mut gog_config = gog_config.unwrap();

    let data_file = fs::canonicalize(format!("./games_data/{id}.json")).await; // This will return error if path doesn't exist
    if let Ok(data_file) = data_file {
        let raw_data = fs::read_to_string(&data_file).await.unwrap();
        if let Ok(config) = serde_json::from_str::<PlatformConfig>(&raw_data) {
            gog_config.content.linux = Some(config);
        };
    }
    (Status::Ok, Some(json::Json(gog_config)))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/api", routes![get_config])
}
