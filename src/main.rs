#[macro_use]
extern crate rocket;
use rocket::{http::Status, serde::json};
use structs::{GOGConfig, PlatformConfig};

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

    match utils::read_linux_config(id).await {
        Some(linux_config) => {
            gog_config.content.linux = Some(linux_config);
        }
        None => gog_config.content.linux = Some(PlatformConfig::default()),
    }

    (Status::Ok, Some(json::Json(gog_config)))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/api", routes![get_config])
}
