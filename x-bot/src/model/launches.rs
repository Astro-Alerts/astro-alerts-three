use chrono::{
    DateTime, Utc, NaiveDateTime,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_repr::{
    Deserialize_repr,
    Serialize_repr,
};

use crate::utils::serde::{
    datetime_formatting,
    // duration,
    // string_option,
};

#[derive(Deserialize, Serialize, Clone)]
pub struct PaginatedPolymorphicLaunchEndpointList {
    pub count: i32,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<PolymorphicLaunchEndpointDetailed>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PolymorphicLaunchEndpointDetailed {
    pub id: String,
    pub url: String,

    pub name: String,
    pub response_mode: String,
    pub slug: String,
    pub launch_designator: Option<String>,

    pub status: Option<Status>,
    pub last_updated: String, // Date
    #[serde(with = "datetime_formatting")]
    pub net: NaiveDateTime, // Date
    pub net_precision: Option<IDStatus>,
    pub window_end: String, // Date
    pub window_start: String, // Date,

    pub image: Option<Image>,
    pub infographic: Option<String>,

    pub probability: Option<i32>,
    pub weather_concerns: Option<String>,
    pub fail_reason: Option<String>,
    pub hashtag: Option<String>,

    pub launch_service_provider: Option<AgencyDetailed>,
    pub rocket: Option<Rocket>,
    pub mission: Option<Mission>,

    pub pad: Option<Pad>,
    pub webcast_live: bool,

    pub orbital_launch_attempt_count: Option<i32>,
    pub location_launch_attempt_count: Option<i32>,
    pub pad_launch_attempt_count: Option<i32>,
    pub agency_launch_attempt_count: Option<i32>,
    pub orbital_launch_attempt_count_year: Option<i32>,
    pub location_launch_attempt_count_year: Option<i32>,
    pub pad_launch_attempt_count_year: Option<i32>,
    pub agency_launch_attempt_count_year: Option<i32>,

    pub info_urls: Vec<InfoURL>,
    pub vid_urls: Vec<VidURL>,

    pub timeline: Vec<TimelineEvent>,
    pub pad_turnaround: String
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Status {
    pub id: LaunchStatus,
    pub name: String,
    pub abbrev: String,
    pub description: String
}

#[derive(Deserialize, Serialize, Clone)]
pub struct IDStatus {
    pub id: i32,
    pub name: String,
    pub abbrev: String,
    pub description: String
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Image {
    pub id: i32,
    pub name: String,

    pub image_url: String,
    pub thumbnail_url: String,
    pub credit: Option<String>,
    pub license: ImageLicense,
    pub single_use: bool,
    pub variants: Vec<ImageVariant>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ImageLicense {
    pub id: i32,
    pub name: String,
    pub priority: i32,
    pub link: Option<String>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ImageVariant {
    pub id: i32,
    #[serde(rename = "type")]
    pub variant_type: Type,
    pub image_url: String
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Type {
    pub id: i32,
    pub name: Option<String>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AgencyDetailed {
    pub response_mode: String,
    pub id: i32,
    pub url: String,

    pub name: String,
    pub abbrev: String,
    pub description: Option<String>,
    pub administrator: Option<String>,

    pub image: Option<Image>,
    pub logo: Option<Image>,
    pub social_logo: Option<Image>,
    pub social_media_links: Vec<SocialMediaLink>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SocialMediaLink {
    pub id: i32,
    pub social_media: SocialMedia,
    pub url: Option<String>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SocialMedia {
    pub id: i32,
    pub name: String,
    pub url: Option<String>,
    pub logo: Option<Image>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Rocket {
    pub id: i32,
    pub configuration: LauncherConfigDetailed,
    pub launcher_stage: Vec<FirstStageNormal>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct LauncherConfigDetailed {
    pub response_mode: String,
    pub id: i32,
    pub url: String,

    pub name: String,
    pub full_name: String,
    pub variant: String
}

#[derive(Deserialize, Serialize, Clone)]
pub struct FirstStageNormal {
    pub id: i32,
    #[serde(rename = "type")]
    pub stage_type: String,
    pub reused: Option<bool>,
    pub launcher_flight_number: Option<i32>,
    pub launcher: LauncherNormal,

    pub previous_flight_date: Option<String>, // Date
    pub turn_around_time: Option<String>,
    pub landing: Option<Landing>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct LauncherNormal {
    pub response_mode: String,
    pub id: i32,
    pub url: String,

    pub flight_proven: bool,
    pub serial_number: Option<String>,
    pub is_placeholder: bool,
    pub status: Option<Type>,
    pub image: Option<Image>,
    pub details: String,

    pub successful_landings: Option<i32>,
    pub attempted_landings: Option<i32>,
    pub flights: Option<i32>,
    
    pub last_launch_date: Option<String>, // Date
    pub first_launch_date: Option<String>, // Date
    pub fastest_turnaround: Option<String>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Landing {
    pub id: i32,
    pub url: String,

    pub attempt: bool,
    pub success: Option<bool>,

    pub description: String,
    pub downrange_distance: Option<f32>,
    pub landing_location: Option<LandingLocation>,
    #[serde(rename = "type")]
    pub landing_type: IDStatus
}

#[derive(Deserialize, Serialize, Clone)]
pub struct LandingLocation {
    pub id: i32,
    pub name: String,

    pub active: bool,
    pub abbrev: String,
    pub description: Option<String>,
    pub image: Option<Image>,

    pub successful_landings: Option<i32>,
    pub attempted_landings: Option<i32>,
    pub failed_landings: Option<i32>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Mission {
    pub id: i32,
    pub name: String,
    #[serde(rename = "type")]
    pub mission_type: String,
    pub description: String,
    pub image: Option<Image>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Pad {
    pub id: i32,
    pub url: String,
    pub active: bool,

    pub name: String,
    pub image: Option<Image>,
    pub description: Option<String>,

    pub info_url: Option<String>,
    pub wiki_url: Option<String>,
    pub map_url: Option<String>,

    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct InfoURL {
    pub priority: i32,
    pub source: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,

    pub feature_image: Option<String>,
    pub url: String,
    #[serde(rename = "type")]
    pub url_type: Option<Type>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct VidURL {
    pub priority: i32,
    pub source: Option<String>,
    pub publisher: Option<String>,

    pub title: Option<String>,
    pub description: Option<String>,
    pub feature_image: Option<String>,
    pub url: String,
    #[serde(rename = "type")]
    pub url_type: Option<Type>,

    pub start_time: Option<String>, // Date
    pub end_time: Option<String>, // Date
    pub live: bool
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TimelineEvent {
    #[serde(rename = "type")]
    pub event_type: Option<Type>,
    pub relative_time: Option<String>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TimelineEventType {
    pub id: i32,
    pub abbrev: String,
    pub description: String
}

////
#[derive(Deserialize, Serialize, Clone)]
pub struct MessageContainer {
    pub message: Option<String>,
    pub launch: PolymorphicLaunchEndpointDetailed
}
////

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum LaunchStatus {
    Go = 1,
    Tbd = 2,
    Success = 3,
    Failure = 4,
    Hold = 5,
    InFlight = 6,
    PartialFailure = 7,
    Tbc = 8,
    PayloadDeployed = 9,
}

impl LaunchStatus {
    pub fn _as_str(&self) -> &str {
        match self {
            LaunchStatus::Go => "Go",
            LaunchStatus::Tbd => "TBD",
            LaunchStatus::Failure => "Failure",
            LaunchStatus::Success => "Success",
            LaunchStatus::InFlight => "In Flight",
            LaunchStatus::Hold => "Hold",
            LaunchStatus::PartialFailure => "Partial Failure",
            LaunchStatus::Tbc => "TBC",
            LaunchStatus::PayloadDeployed => "Payload Deployed"
        }
    }
}
