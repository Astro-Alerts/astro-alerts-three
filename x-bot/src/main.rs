use critter::{ TwitterClient, auth::TwitterAuth };
use redis::Client;
use utils::env;

mod bot;
mod utils;
mod model;

use model::launches;
use model::launches::LaunchStatus;
use bot::bot::notify_message;
// use bot::status::{get_updates, TrackingStatus};

// Function to get the ordinal suffix after a number
fn ordinal_suffix(n: u32) -> String {
    let suffix = match n % 100 {
        11 | 12 | 13 => "th",
        _ => match n % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };
    format!("{}{}", n, suffix)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // std::env::set_var("RUST_LOG", "debug");
    // std::env::set_var("RUST_BACKTRACE", "1");

    utils::env::init();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("starting x bot");

    let auth: TwitterAuth = TwitterAuth::from_oa1uc(
        &env::api_key(),
        &env::api_key_secret(),
        &env::access_token(),
        &env::access_token_secret()
    );

    let twitter: TwitterClient = TwitterClient::new(auth).unwrap();
    log::info!("Astro Alerts x-bot is online");

    let redis_client = Client::open("redis://127.0.0.1/").expect("Failed to create Redis Client");
    let mut binding = redis_client.get_connection().expect("Failed to create Redis Connection");
    let mut redis_connection = binding.as_pubsub();
    log::info!("redis connection online");

    // Subscribe to all of the relevant channels to receive information from the API
    redis_connection.subscribe("new_launches").expect("Failed to subscribe to `new_launches` channel");
    redis_connection.subscribe("scrubbed_launches").expect("Failed to subscribe to scrubbed_launches` channel");
    redis_connection.subscribe("updated_launches").expect("Failed to subscribe to `updated_launches` channel");
    redis_connection.subscribe("launch_time_left").expect("Failed to subscribe to `launch_time_left` channel");
    log::info!("subscribed to all channels");

    // Main loop, receive messages and act upon the data received
    loop {
        // Get latest message
        let msg = match redis_connection.get_message() {
            Ok(v) => v,
            Err(_) => {
                log::error!("failed to get redis message");
                continue
            }
        };

        // Get channel name (type of event)
        let channel_name: String = match msg.get_channel() {
            Ok(v) => v,
            Err(_) => {
                log::error!("failed to get redis channel");
                continue
            }
        };

        // Get the message received
        let payload: String = match msg.get_payload() {
            Ok(v) => v,
            Err(_) => {
                log::error!("failed to get redis payload");
                continue
            }
        };
 
        // Deserialize the message received to get the message container and the associated data
        let message_container = match serde_json::from_str::<launches::MessageContainer>(&payload) {
            Ok(v) => v,
            Err(_) => {
                log::error!("failed to deserialize launch");
                continue
            }
        };

        // Prepare the information the bot needs to act upon the data
        let launch_info: launches::PolymorphicLaunchEndpointDetailed = message_container.launch;
        log::info!("📩 received message from channel {} regarding launch {}", channel_name, launch_info.name);

        let launch_status = match launch_info.status {
            Some(v) => v,
            None => continue
        };
    
        let vehicle = match launch_info.rocket {
            Some(v) => v,
            None => continue
        };
    
        let mission = match launch_info.mission {
            Some(v) => v,
            None => continue
        };
    
        let pad = match launch_info.pad {
            Some(v) => v,
            None => continue
        };
    
        let lsp = match launch_info.launch_service_provider {
            Some(v) => v,
            None => continue
        };

        // New launch message
        if channel_name == "new_launches" {
            let message: &String = &format!(
                "{} is GO to launch {} on {} at {} UTC! The mission: {} 🟢",
                &vehicle.configuration.full_name,
                &mission.name,
                launch_info.net.date().format("%d-%m-%Y"),
                launch_info.net.time(),
                mission.description
            );

            notify_message(message, &launch_info.image, &launch_info.id, twitter.clone()).await;
        }
        
        // Scrubbed launch message
        if channel_name == "scrubbed_launches" {
            let message: &String = &format!(
                "The {} launch of {} has been SCRUBBED! The launch is now scheduled for {} at {}.",
                vehicle.configuration.full_name,
                mission.name,
                launch_info.net.date().format("%d-%m-%Y"),
                if launch_status.id == LaunchStatus::Tbd {
                    " (To Be Determined)".to_owned()
                } else {
                    launch_info.net.time().to_string() + " UTC"
                }
            );

            notify_message(message, &launch_info.image, &launch_info.id, twitter.clone()).await;
        }
        
        // Updated launch message
        if channel_name == "updated_launches" {
            // Go through every status we'd like to send a message for
            match launch_status.id {
                LaunchStatus::Go => {
                    let message: &String = &format!(
                        "{} is GO to launch {} on {} at {} UTC! The mission: {} 🟢",
                        &vehicle.configuration.full_name,
                        &mission.name,
                        launch_info.net.date().format("%d-%m-%Y"),
                        launch_info.net.time(),
                        mission.description
                    );
        
                    notify_message(message, &launch_info.image, &launch_info.id, twitter.clone()).await;
                },
                LaunchStatus::Hold => {
                    let message: &String = &format!(
                        "The launch of {} on board {} is on HOLD! 🟡",
                        &mission.name,
                        &vehicle.configuration.full_name,
                    );
        
                    notify_message(message, &None, &launch_info.id, twitter.clone()).await;
                },
                LaunchStatus::InFlight => {
                    let message: &String = &format!(
                        "LIFTOFF of {}'s {} from {} carrying {} to space! 🚀",
                        &lsp.name,
                        &vehicle.configuration.full_name,
                        &pad.name,
                        &mission.name
                    );
                
                    notify_message(message, &launch_info.image, &launch_info.id, twitter.clone()).await;
                },
                LaunchStatus::Success => {
                    let message: &String = &format!(
                        "{} SUCCESSFULLY launched {} to space from {} on board {}! 🛰️",
                        &lsp.name,
                        &mission.name,
                        &pad.name,
                        &vehicle.configuration.full_name,
                    );
                
                    notify_message(message, &None, &launch_info.id, twitter.clone()).await;
                },
                LaunchStatus::PayloadDeployed => {
                    let message: &String = &format!(
                        "{} has been DEPLOYED from {}! 🛰️",
                        &mission.name,
                        &vehicle.configuration.full_name,
                    );
                
                    notify_message(message, &None, &launch_info.id, twitter.clone()).await;
                },
                LaunchStatus::PartialFailure => {
                    let message: &String = &format!(
                        "{} suffered a PARTIAL FAILIURE during their launch of {} from {} on board {} to space. 🛰️",
                        &lsp.name,
                        &mission.name,
                        &pad.name,
                        &vehicle.configuration.full_name,
                    );
                
                    notify_message(message, &None, &launch_info.id, twitter.clone()).await;
                },
                LaunchStatus::Failure => {
                    let message: &String = &format!(
                        "{} suffered a FAILIURE during their launch of {} from {} on board {} to space. 🛰️",
                        &lsp.name,
                        &mission.name,
                        &pad.name,
                        &vehicle.configuration.full_name,
                    );
                
                    notify_message(message, &None, &launch_info.id, twitter.clone()).await;
                }
                LaunchStatus::Tbd => (),
                LaunchStatus::Tbc => ()
            }
        }

        // Launch Time Left message
        if channel_name == "launch_time_left" {
            let time_left: String = match message_container.message {
                Some(v) => v,
                None => continue
            };

            if launch_info.vid_urls.len() == 0 {
                // No live webcast was provided

                let message: &String = &format!(
                    "{}! {} is GO to launch {} from {}. 🚀",
                    &time_left,
                    &vehicle.configuration.full_name,
                    &mission.name,
                    &pad.name,
                );

                notify_message(message, &None, &launch_info.id, twitter.clone()).await;
            } else {
                // Live webcast is provided, get the one with the highest priority (usually the official webcast)

                let mut highest_priority: i32 = 0;
                let mut selected_video_url: String = "".to_string();
                for vid in &launch_info.vid_urls {
                    if vid.priority > highest_priority {
                        highest_priority = vid.priority;
                        selected_video_url = vid.url.clone();
                    }
                }

                if !selected_video_url.is_empty() {
                    let message: &String = &format!(
                        "{}! {} is GO to launch {} from {}. 🚀\nWATCH LIVE: {}",
                        &time_left,
                        &vehicle.configuration.full_name,
                        &mission.name,
                        &pad.name,
                        &selected_video_url,
                    );

                    notify_message(message, &None, &launch_info.id, twitter.clone()).await;
                }
            }
        }

        // Booster Info message (for reusable boosters)
        if channel_name == "booster_info" {
            let message: &String = &format!(
                "The booster supporting today's {} mission{} will be flying for its {}. {}.",
                vehicle.configuration.name,
                match &vehicle.launcher_stage[0].launcher.serial_number {
                    Some(v) => " (".to_owned() + &v + ")",
                    None => "".to_owned()
                },
                match &vehicle.launcher_stage[0].launcher_flight_number {
                    Some(v) => {
                        if vehicle.launcher_stage[0].launcher.flight_proven { 
                            ordinal_suffix(v.to_owned() as u32) + " time having previously supported " + &(v - 1).to_string() + " missions"
                        } else {
                            "first time".to_owned()
                        }
                    },
                    None => "first time".to_owned()
                },
                if let Some(landing) = &vehicle.launcher_stage[0].landing {
                    if landing.attempt {
                        if vehicle.configuration.name == "Falcon 9" || vehicle.configuration.name == "New Glenn" || vehicle.configuration.name == "New Shepard" {
                            match &landing.landing_location {
                                Some(v) => "The booster will attempt to land on/at ".to_owned() + &v.name + " for recovery and reuse in future missions",
                                None => "The booster will attempt to land for recovery and reuse in future missions".to_owned()
                            }
                        } else if vehicle.configuration.name == "Starship" {
                            match &landing.landing_location {
                                Some(v) => "The booster will perform a catch attempt using Mechazilla's tower arms at ".to_owned() + &v.name + ". After recovery, the booster will be closely inspected in order to gather as much data possible. This will help the future development of Starship.",
                                None => "The booster will perform a catch attempt using Mechazilla's tower arms for recovery and use in future missions".to_owned()
                            }
                        } else {
                            continue
                        }
                    } else {
                        "No recovery attempt will be carried out for this booster, therefore it will be flying for its last time and expended after completing its final mission".to_owned()
                    }
                } else {
                    "No recovery attempt will be carried out for this booster, therefore it will be flying for its last time and expended after completing its final mission".to_owned()
                }
            );

            notify_message(message, &vehicle.launcher_stage[0].launcher.image, &launch_info.id, twitter.clone()).await;
        }
    }

    // match twitter.me(None).await {
    //     Ok(data) => log::info!("{} is online", data.name()),
    //     Err(e) => {
    //         log::error!("Error: {}", e);
    //         panic!()
    //     }
    // }

    // let mut status: TrackingStatus = TrackingStatus { cache: get_updates().await.unwrap(), twitter };

    // let mut interval: tokio::time::Interval =
    //     tokio::time::interval(tokio::time::Duration::from_secs(60));

    // interval.tick().await;

    // loop {
    //     interval.tick().await;
    //     status.check_for_updates().await;
    // }

    // Ok(())
}
