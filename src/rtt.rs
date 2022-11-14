use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub(crate) enum MultiTiploc {
    Single(String),
    Multi(Vec<String>),
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Location {
    // pub(crate) name: String,
    // pub(crate) crs: String,
    pub(crate) tiploc: String,
    // country: String,
    // system: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ServiceLocation {
    pub(crate) tiploc: String,
    pub(crate) description: String,
    // workingTime: String,
    pub(crate) public_time: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ServiceLocationDetail {
    // realtimeActivated: bool,
    // tiploc: String,
    // crs: String,
    pub(crate) description: String,

    // pub(crate) gbtt_booked_arrival: Option<String>,
    // pub(crate) gbtt_booked_departure: Option<String>,
    // pub(crate) origin: Vec<ServiceLocation>,
    pub(crate) destination: Vec<ServiceLocation>,
    
    // isCall: bool,
    // isPublicCall: bool,
    pub(crate) realtime_arrival: Option<String>,
    pub(crate) realtime_arrival_actual: Option<bool>,

    pub(crate) realtime_departure: Option<String>,
    pub(crate) realtime_departure_actual: Option<bool>,

    pub(crate) realtime_gbtt_arrival_lateness: Option<i32>,
    pub(crate) realtime_gbtt_departure_lateness: Option<i32>,
    pub(crate) platform: Option<String>,
    // pub(crate) platform_confirmed: Option<bool>,
    pub(crate) platform_changed: Option<bool>,
    // pub(crate) path: Option<String>,
    // pub(crate) path_confirmed: Option<bool>,
    // pub(crate) line: Option<String>,
    pub(crate) service_location: Option<String>,
    pub(crate) display_as: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LocationService {
    pub(crate) location_detail: ServiceLocationDetail,
    pub(crate) service_uid: String,
    
    // runDate: String, // "2022-11-10"
    // trainIdentity: String, // "1L75"
    // runningIdentity: String, // "1L75"
    // atocCode: String, // "SR"
    // serviceType: String, // "train"
    // isPassenger: bool,
}

#[derive(Deserialize)]
pub(crate) struct LocationSearchResponse {
    pub(crate) location: Location,
    pub(crate) services: Vec<LocationService>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ServiceResponse {
    pub(crate) atoc_name: String,
    pub(crate) origin: Vec<ServiceLocation>,
    pub(crate) destination: Vec<ServiceLocation>,
    pub(crate) locations: Vec<ServiceLocationDetail>,
}
