use crate::utils::constants::MONGO_REPOSITORY;

pub async fn clean_launch_data() {
    log::info!("purging data");
    
    let database = match MONGO_REPOSITORY.get() {
        Some(v) => v,
        None => {
            log::error!("failed to get database while purging launch data");
            return;
        }
    };
    
    match database.purge_launch_data().await {
        Ok(_) => log::info!("called for data purge"),
        Err(_) => log::error!("failed to purge data")
    };
}
