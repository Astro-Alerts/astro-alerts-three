use critter::{TwitterClient, TwitterMediaResponse};
use tokio::time::{sleep, Duration};
use std::path::{Path, PathBuf};
use crate::model::launches::Image;

async fn get_image(id: &String, image: &Image, mut twitter: TwitterClient) -> (Option<TwitterMediaResponse>, Option<PathBuf>) {
    let mut pic = None;

    let image_name: &String = &format!("./{}-image.jpg", id);
    let image_path: &Path = Path::new(image_name);

    let image_url: &String = &image.image_url;

    let mut image_response_data = match reqwest::get(image_url)
        .await {
            Ok(v) => v,
            Err(e) => {
                log::error!("pic error 0x2 {}", e);
                return (None, None)
            }
        };
    
    image_response_data = match image_response_data.error_for_status() {
        Ok(v) => v,
        Err(e) => {
            log::error!("pic error 0x3 {}", e);
            return (None, None)
        }
    };

    let bytes = match image_response_data.bytes().await {
        Ok(v) => v,
        Err(e) => {
            log::error!("pic error 0x4 {}", e);
            return (None, None)
        }
    };

    let mut dest = match std::fs::File::create(image_path) {
        Ok(v) => v,
        Err(e) => {
            log::error!("pic error 0x5 {}", e);
            return (None, None)
        }
    };

    let mut content = std::io::Cursor::new(bytes);
    std::io::copy(&mut content, &mut dest).unwrap();

    match image_path.to_str() {
        Some(v) => {
            pic = match twitter.upload_media(v, Some("image.jpg".into())).await {
                Ok(pic) => Some(pic),
                Err(e) => {
                    log::info!("Error uploading media: {}", e);
                    None
                },
            };
        },
        None => {
            log::error!("pic error 0x6");
            return (None, None)
        }
    };

    return (pic, Some(image_path.to_path_buf()))
}

async fn post(message: &String, image: Option<TwitterMediaResponse>, path: Option<PathBuf>, mut twitter: TwitterClient) {
    let image_exists: bool = image.is_some();

    if image_exists {
        match twitter.tweet(|tweet|
            tweet.text(&message)
            .media(|m| {
                m.add(image)
            })
        ).await {
            Ok(data) => log::info!("Posted a post with ID of: {:?}", data.id()),
            Err(e) => log::info!("X Error 0x1: {}", e)
        };
    } else {
        match twitter.tweet(|tweet|
            tweet.text(&message)
        ).await {
            Ok(data) => log::info!("Posted a post with ID of: {:?}", data.id()),
            Err(e) => log::info!("X Error 0x2: {}", e)
        };
    }
    
    if image_exists {
        match path {
            Some(v) => {
                match std::fs::remove_file(&v) {
                    Ok(_) => (),
                    Err(_) => log::error!("failed to delete image {}", v.to_string_lossy())
                };
            },
            None => return
        };
    }
}

pub async fn notify_message(message: &String, image: &Option<Image>, id: &String, twitter: TwitterClient) {
    sleep(Duration::from_secs(10)).await;

    if let Some(img) = image {
        let (pic, path) = get_image(id, &img, twitter.clone()).await;

        let picture = match pic {
            Some(v) => v,
            None => return post(message, None, None, twitter).await
        };

        let picture_path = match path {
            Some(v) => v,
            None => return post(message, None, None, twitter).await
        };

        return post(message,Some(picture), Some(picture_path), twitter).await;
    }

    post(message, None, None, twitter).await
}
