use deadpool_redis::{Pool, redis::AsyncCommands};
use chrono::{TimeDelta, Utc};
use std::sync::Arc;
use crate::{
    model::launches::{
        LaunchStatus, MessageContainer, PaginatedPolymorphicLaunchEndpointList, PolymorphicLaunchEndpointDetailed
    },
    utils::constants::MONGO_REPOSITORY,
};

#[derive(Debug)]
pub enum TimeLeft {
    FIVEHOURS,
    FOURHOURS,
    THREEHOURS,
    TWOHOURS,
    ONEHOUR,

    THIRTY,
    FIFTEEN,
    TEN,
    FIVE
}

impl TimeLeft {
    pub fn as_str(&self) -> &str {
        match self {
            TimeLeft::FIVEHOURS => "T-5 Hours",
            TimeLeft::FOURHOURS => "T-4 Hours",
            TimeLeft::THREEHOURS => "T-3 Hours",
            TimeLeft::TWOHOURS => "T-2 Hours",
            TimeLeft::ONEHOUR => "T-1 Hour",

            TimeLeft::THIRTY => "T-30 minutes",
            TimeLeft::FIFTEEN => "T-15 minutes",
            TimeLeft::TEN => "T-10 minutes",
            TimeLeft::FIVE => "T-5 minutes",
        }
    }
}

async fn send_time_left(redis_pool: &Arc<Pool>, launch: &PolymorphicLaunchEndpointDetailed, time_left: TimeLeft) {
    let mut redis_connection = match redis_pool.get().await {
        Ok(v) => v,
        Err(_) => {
            log::error!("failed to get redis pool");
            return;
        }
    };

    let time_data = MessageContainer {
        message: Some(time_left.as_str().to_owned()),
        launch: launch.to_owned()
    };

    let update_json: String = match serde_json::to_string(&time_data) {
        Ok(v) => v,
        Err(_) => return
    };

    let _: String = match redis_connection.publish("launch_time_left", update_json).await {
        Ok(v) => {
            log::info!("sent launch time left to redis");
            v
        },
        Err(_) => {
            log::error!("failed to send launch time left to redis");
            return
        }
    };
}

pub async fn check_time_left_to_launch(redis_pool: &Arc<Pool>) {
    log::info!("checking time left to launches");

    let database = match MONGO_REPOSITORY.get() {
        Some(v) => v,
        None => {
            log::error!("failed to get database while updating launch data");
            return;
        }
    };

    let launches: PaginatedPolymorphicLaunchEndpointList = match database.list_launch_data().await {
        Ok(v) => v,
        Err(_) => {
            log::error!("failed to get launch data from the database");
            return;
        }
    };

    for launch in launches.results {
        let launch_status = match launch.clone().status {
            Some(v) => v,
            None => {
                log::error!("failed to get launch status for launch {}", launch.name);
                continue;
            }
        };

        let vehicle = match &launch.rocket {
            Some(v) => v,
            None => {
                log::error!("failed to get vehicle during status check for launch {}", launch.name);
                continue;
            }
        };

        let net_diff_5: TimeDelta = launch.net.time() - Utc::now().time();
        let minutes_delta: i64 = net_diff_5.num_minutes();

        if launch.net.date() == Utc::now().date_naive() {
            // Time Alerts
            if launch_status.id == LaunchStatus::Go && (minutes_delta <= 31 && minutes_delta > 30) {
                send_time_left(redis_pool, &launch, TimeLeft::THIRTY).await;
            }

            if launch_status.id == LaunchStatus::Go && (minutes_delta <= 16 && minutes_delta > 15) {
                send_time_left(redis_pool, &launch, TimeLeft::FIFTEEN).await;
            }

            if launch_status.id == LaunchStatus::Go && (minutes_delta <= 11 && minutes_delta > 10) {
                send_time_left(redis_pool, &launch, TimeLeft::TEN).await;
            }

            if launch_status.id == LaunchStatus::Go && (minutes_delta <= 6 && minutes_delta > 5) {
                send_time_left(redis_pool, &launch, TimeLeft::FIVE).await;
            }

            // Starship Extra Time Alerts (big event many people follow)
            if vehicle.configuration.name == "Starship" {
                // 5 Hours
                if launch_status.id == LaunchStatus::Go && (minutes_delta <= 301 && minutes_delta > 300) {
                    send_time_left(redis_pool, &launch, TimeLeft::FIVEHOURS).await;
                }

                // 4 Hours
                if launch_status.id == LaunchStatus::Go && (minutes_delta <= 241 && minutes_delta > 240) {
                    send_time_left(redis_pool, &launch, TimeLeft::FOURHOURS).await;
                }

                // 3 Hours
                if launch_status.id == LaunchStatus::Go && (minutes_delta <= 181 && minutes_delta > 180) {
                    send_time_left(redis_pool, &launch, TimeLeft::THREEHOURS).await;
                }

                // 2 Hours
                if launch_status.id == LaunchStatus::Go && (minutes_delta <= 121 && minutes_delta > 120) {
                    send_time_left(redis_pool, &launch, TimeLeft::TWOHOURS).await;
                }

                // 1 Hour
                if launch_status.id == LaunchStatus::Go && (minutes_delta <= 61 && minutes_delta > 60) {
                    send_time_left(redis_pool, &launch, TimeLeft::ONEHOUR).await;
                }
            }
            
            // Misc Alerts based on time
            if launch_status.id == LaunchStatus::Go && (minutes_delta <= 8 && minutes_delta > 7 ) {
                if (vehicle.configuration.name == "Falcon 9" || vehicle.configuration.name == "Falcon Heavy" || vehicle.configuration.name == "Starship" || vehicle.configuration.name == "New Shepard" || vehicle.configuration.name == "New Glenn") && vehicle.launcher_stage.len() > 0 {
                    let mut redis_connection = match redis_pool.get().await {
                        Ok(v) => v,
                        Err(_) => {
                            log::error!("failed to get redis pool");
                            return;
                        }
                    };

                    let booster_data: MessageContainer = MessageContainer {
                        message: None,
                        launch: launch.to_owned()
                    };

                    let update_json: String = match serde_json::to_string(&booster_data) {
                        Ok(v) => v,
                        Err(_) => return
                    };

                    let _: String = match redis_connection.publish("booster_info", update_json).await {
                        Ok(v) => {
                            log::info!("sent launch time left to redis");
                            v
                        },
                        Err(_) => {
                            log::error!("failed to send launch time left to redis");
                            return
                        }
                    };
                }
            }
        }
    }
}
