use std::collections::HashMap;
use deadpool_redis::{Pool, redis::AsyncCommands};
use reqwest::Result;
use std::sync::Arc;
// use tokio::fs::File;
// use tokio::io::AsyncWriteExt;
use crate::{
    model::launches::{
        LaunchStatus, MessageContainer, PaginatedPolymorphicLaunchEndpointList, PolymorphicLaunchEndpointDetailed
    },
    utils::constants::{
        DEFAULT_CLIENT,
        MONGO_REPOSITORY,
    },
};

pub async fn update_data(redis_pool: &Arc<Pool>) {
    log::info!("updating data");

    let mut redis_connection = match redis_pool.get().await {
        Ok(v) => v,
        Err(_) => {
            log::error!("failed to get redis pool");
            return;
        }
    };

    let database = match MONGO_REPOSITORY.get() {
        Some(v) => v,
        None => {
            log::error!("failed to get database while updating launch data");
            return;
        }
    };

    // let mut launches: Vec<LaunchData> = match get_upcoming_launches().await {
    //     Ok(ls) => {
    //         ls.results
    //     },
    //     Err(e) => {
    //         log::error!("failed to fetch new launch data {}", e);
    //         return;
    //     },
    // };

    let launches_container = match get_upcoming_launches().await {
        Ok(v) => v,
        Err(_) => {
            log::error!("failed to get upcoming launches");
            return;
        }
    };

    let mut launches: Vec<PolymorphicLaunchEndpointDetailed> = launches_container.results;
    launches.sort_by_key(|l| l.net.clone());

    log::info!("fetched {} launches", launches.len());

    for launch in launches {
        //log::info!("name {}", &launch.name);

        // Filter launch status, do not want to add launches that are not confirmed
        match &launch.status {
            Some(v) => {
                if v.id == LaunchStatus::Tbd { continue; }
                if v.id == LaunchStatus::Tbc { continue; }
            },
            None => continue
        };

        // Filter launch service provider, do not want updates from these providers
        match &launch.launch_service_provider {
            Some(v) => {
                if v.abbrev == "CASC" { continue; }
                if v.abbrev == "MHI" { continue; }
                if v.abbrev == "Space One" { continue; }
                if v.abbrev == "ISRO" { continue; }
                if v.abbrev == "CAS" { continue; }
                if v.abbrev == "GE" { continue; }
            },
            None => continue
        };

        match database.get_launch_from_id(&launch.id).await {
            Some(v) => {
                // Launch already exists in database, update the data
                if database.update_launch_data(&launch, &v, &mut redis_connection).await.is_err() {
                    log::error!("failed to update launch {} data", &launch.id);
                };
            }
            None => {
                // Launch is new to us, add it to the database
                match database.add_launch(&launch).await {
                    Ok(_) => log::info!("added launch {} to the database", &launch.id),
                    Err(_) => {
                        log::error!("failed to add launch {} to the database", &launch.id);
                        return;
                    }
                };
                
                let launch_status  = match &launch.status {
                    Some(v) => v,
                    None => return
                };

                // If the launch is go, send it to the new_launches channel
                if launch_status.id == LaunchStatus::Go {
                    let update_data = MessageContainer {
                        message: None,
                        launch: launch
                    };

                    let update_json: String = match serde_json::to_string(&update_data) {
                        Ok(v) => v,
                        Err(_) => return
                    };

                    let _: String = match redis_connection.publish("new_launches", update_json).await {
                        Ok(v) => {
                            log::info!("sent new launch to redis");
                            v
                        },
                        Err(_) => {
                            log::error!("failed to send new launch to redis");
                            return
                        }
                    };

                    // let _: String = match redis_connection.xadd("new_launch", "*", &[("data", update_json.as_str())]) {
                    //     Ok(v) => {
                    //         log::info!("sent new launch to redis streams with id {}", v);
                    //         v
                    //     },
                    //     Err(_) => {
                    //         log::error!("failed to send new launch to redis streams");
                    //         return
                    //     }
                    // };
                }
            }
        };
    }
}

async fn get_upcoming_launches() -> Result<PaginatedPolymorphicLaunchEndpointList> {
    let mut params = HashMap::new();
    params.insert("limit", "100");
    params.insert("format", "json");
    params.insert("mode", "detailed");

    // let content = DEFAULT_CLIENT.get("https://lldev.thespacedevs.com/2.3.0/launches/upcoming/").query(&params).send().await?.bytes().await?;

    // Open the file and write the content to it
    // let mut file = tokio::fs::File::create("output.txt").await.unwrap();
    // file.write_all(&content).await.unwrap();

    DEFAULT_CLIENT
        .get("https://ll.thespacedevs.com/2.3.0/launches/upcoming/")
        .query(&params)
        .send()
        .await?
        .error_for_status()?
        .json::<PaginatedPolymorphicLaunchEndpointList>()
        .await
}
