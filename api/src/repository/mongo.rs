use mongodb::{bson, options::ClientOptions, Client, Collection, Database};
use futures::stream::TryStreamExt;
use chrono::{Duration, Utc};
use deadpool_redis::{Connection, redis::AsyncCommands};

use crate::model::launches::{PolymorphicLaunchEndpointDetailed, PaginatedPolymorphicLaunchEndpointList, MessageContainer};

pub struct MongoRepository {
    _client: Client,
    _db: Database,
    launch_collection: Collection<PolymorphicLaunchEndpointDetailed>,
}

pub struct MongoError;

impl MongoRepository {
    pub fn init(client_options: ClientOptions) -> MongoRepository {
        let _client: Client =
            Client::with_options(client_options).expect("Could not open a handle to the database");
        let _db: Database = _client.database("astro_alerts_api");

        let launch_collection: Collection<PolymorphicLaunchEndpointDetailed> = _db.collection::<PolymorphicLaunchEndpointDetailed>("launches");

        MongoRepository {
            _client,
            _db,
            launch_collection
        }
    }

    //////////

    pub async fn add_launch(&self, launch_info: &PolymorphicLaunchEndpointDetailed) -> Result<(), MongoError> {
        match self.launch_collection.insert_one(launch_info).await {
            Ok(_) => Ok(()),
            Err(_) => Err(MongoError),
        }
    }

    pub async fn get_launch_from_id(&self, launch_id: &String) -> Option<PolymorphicLaunchEndpointDetailed> {
        let filter: bson::Document = bson::doc! { "id": launch_id };
        let found = self.launch_collection.find_one(filter).await;

        match found {
            Ok(output) => output,
            Err(_) => None,
        }
    }

    pub async fn update_launch_data(&self, new_launch_info: &PolymorphicLaunchEndpointDetailed, current_launch_info: &PolymorphicLaunchEndpointDetailed, redis_connection: &mut Connection) -> Result<(), MongoError> {
        let new_launch_status = match &new_launch_info.status {
            Some(v) => v,
            None => return Ok(())
        };

        let current_launch_status = match &current_launch_info.status {
            Some(v) => v,
            None => return Ok(())
        };

        // If the launch status has not changed and the launch NET date has not changed, no need to update the launch info
        if (new_launch_status.id == current_launch_status.id) & (new_launch_info.net == current_launch_info.net) {
            return Ok(());
        }

        // Launch status or NET date has changed, update the launch info
        log::info!("updating launch data in database");

        // Delete the previous document
        let filter: bson::Document = bson::doc! { "id": &new_launch_info.id };
        let deleted = self
            .launch_collection
            .delete_one(filter)
            .await;
        
        // If we failed to delete the previous record, return and send no messages
        if deleted.is_err() {
            log::error!("failed to delete old launch data to replace with updated data");
            return Err(MongoError)
        }

        // If we failed to insert the new record, return and send no messages
        if self.add_launch(new_launch_info).await.is_err() {
            log::error!("failed to insert the new launch data");
            return Err(MongoError)
        }

        let update_data = MessageContainer {
            message: None,
            launch: new_launch_info.to_owned()
        };

        let update_json: String = match serde_json::to_string(&update_data) {
            Ok(v) => v,
            Err(_) => return Err(MongoError)
        };

        let minutes = Duration::minutes(30);
        if new_launch_info.net > (current_launch_info.net + minutes) {
            // Launch has been scrubbed, send the info on the scrubbed_launches channel
            let _: String = match redis_connection.publish("scrubbed_launches", update_json).await {
                Ok(v) => {
                    log::info!("sent launch scrub to redis");
                    v
                },
                Err(_) => {
                    log::error!("failed to send launch scrub to redis");
                    return Err(MongoError)
                }
            };
        } else {
            // Send the updated launch info on the updated_launches channel
            let _: String = match redis_connection.publish("updated_launches", update_json).await {
                Ok(v) => {
                    log::info!("sent updated launch to redis");
                    v
                },
                Err(_) => {
                    log::error!("failed to send updated launch to redis");
                    return Err(MongoError)
                }
            };
        }

        return Ok(())
    }

    pub async fn list_launch_data(&self) -> Result<PaginatedPolymorphicLaunchEndpointList, MongoError> {
        let cursor = self.launch_collection.find(bson::doc! {}).await;
        match cursor {
            Ok(mut v) => {
                let mut results: Vec<PolymorphicLaunchEndpointDetailed> = Vec::new();

                while let Ok(Some(result)) = v.try_next().await {
                    results.push(result);
                }

                let a = PaginatedPolymorphicLaunchEndpointList { count: results.len() as i32, next: None, previous: None, results };
                Ok(a)
            },
            Err(_) => Err(MongoError)
        }
    }

    pub async fn purge_launch_data(&self) -> Result<(), MongoError> {
        // Purge launch documents that have a NET date that was 7 days ago or greater
        
        let purged = self.launch_collection.delete_many(bson::doc! { "net": { "$lt" : (Utc::now() - Duration::days(7)).to_rfc3339() } }).await;
        match purged {
            Ok(_) => Ok(()),
            Err(_) => Err(MongoError)
        }
    }
}
