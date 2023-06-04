use anyhow::bail;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::{fs::{File, self}, time::{Duration, Instant}, path::Path};

use crate::rtt;
use rtt::StringOrStringVec;

#[derive(Deserialize)]
struct ConfigFile {
    username: String,
    password: String,
}

const API_BASE: &str = "https://api.rtt.io/api/v1/json";
const DEBUG_DIR: &str = "~/.rtttui/debug";

pub(crate) struct App {
    destination: Option<rtt::Location>,
    location_service: Option<rtt::LocationService>,
    pub(crate) service: Option<rtt::ServiceResponse>,

    pub(crate) show_intermediary: bool,

    pub(crate) should_quit: bool,
    pub(crate) last_service_update: Option<Instant>,
    refresh_rate: Duration,
    debug: bool,

    config: ConfigFile,
    http: reqwest::Client,
}

impl App {
    pub fn new(refresh_rate: Duration, debug: bool) -> App{
        let cfg_path = shellexpand::tilde("~/.config/rtt.yaml").to_string();
        let cfg_file = File::open(cfg_path).unwrap();
        let config: ConfigFile = serde_yaml::from_reader(&cfg_file).unwrap();

        let debug_path = shellexpand::tilde(DEBUG_DIR).to_string();
        let debug_dir = Path::new(&debug_path);

        if debug {
            if !debug_dir.is_dir() {
                fs::create_dir_all(debug_dir).expect("Unable to create debug output dir");
            }
        }

        App {
            destination: None,
            location_service: None,
            service: None,

            show_intermediary: false,

            should_quit: false,
            last_service_update: None,
            refresh_rate,
            debug,

            config,
            http: reqwest::Client::new(),
        }
    }

    pub fn on_key(&mut self, c: char) {
        // We're allowing single_match here because we're assuming that we'll likely
        // want to support other keys in future
        #[allow(clippy::single_match)]
        match c {
            'i' => self.show_intermediary = !self.show_intermediary,
            'q' => self.should_quit = true,
            _ => {}
        }
    }

    pub async fn load_destination(&mut self, dest: String) -> anyhow::Result<()> {
        let url = format!("{}/search/{}", API_BASE, dest);
        let resp = self.http
            .get(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await?;

        if !resp.status().is_success() {
            bail!("Unable to get destination station: {}", resp.status());
        }

        let raw_response = resp.text().await?;

        self.destination = match serde_json::from_str::<rtt::LocationSearchResponse>(&raw_response) {
            Ok(parsed) => Some(parsed.location),
            Err(err) => {
                fs::write("destination.json", raw_response).expect("Unable to dump destination.json");
                bail!("Unable to parse destination station: {} {}", err, url)
            }
        };
        
        Ok(())
    }

    pub async fn find_service(&mut self, source: String, departure_time: String) -> anyhow::Result<()> {
        let now: DateTime<Utc> = Utc::now();
        let url = format!(
            "{}/search/{}/{}/{}",
            API_BASE,
            source,
            now.format("%Y/%m/%d"),
            departure_time,
        );
        let resp = self.http
            .get(url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await?;
        
        if !resp.status().is_success() {
            bail!("Unable to get source: {}", resp.status());
        }
        
        let raw_response = resp.text().await?;

        let location_search: rtt::LocationSearchResponse = match serde_json::from_str(&raw_response) {
            Ok(data) => data,
            Err(err) => {
                fs::write("source.json", raw_response).expect("Unable to dump source.json");
                bail!("Unable to parse source: {}", err);
            }
        };

        let destination_tiploc = match &self.destination.as_ref().unwrap().tiploc {
            StringOrStringVec::String(tiploc) => tiploc,
            StringOrStringVec::Vec(tiplocs) => &tiplocs[0]
        };

        let filtered_services: Vec<rtt::LocationService> = location_search.services
            .into_iter()
            .filter(|service: &rtt::LocationService| &service.location_detail.destination.first().unwrap().tiploc == destination_tiploc)
            .collect::<Vec<rtt::LocationService>>();
    
        let mut found_closeness: i16 = 999;

        let search_departure_time: i16 = departure_time.parse().expect("Unable to parse departure time as a number");

        for service in filtered_services {
            if service.location_detail.gbtt_booked_departure.is_none() {
                continue;
            }

            let gbtt_booked_departure = service
                .location_detail
                .gbtt_booked_departure
                .as_ref();
            
            let service_booked_departure: i16 = gbtt_booked_departure
                .unwrap_or(&String::from("0"))
                .parse::<i16>()
                .expect("Unable to parse service departure");

            
            let closeness = (search_departure_time - service_booked_departure).abs();
            if closeness < found_closeness {
                self.location_service = Some(service.clone());
                found_closeness = closeness;
            }
        }

        Ok(())
    }

    pub async fn refresh_service(&mut self) -> anyhow::Result<()> {
        let now: DateTime<Utc> = Utc::now();
        let url = format!(
            "{}/service/{}/{}",
            API_BASE,
            self.location_service.as_ref().unwrap().service_uid,
            now.format("%Y/%m/%d"),
        );

        let resp = self.http
            .get(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await?;
        
        if !resp.status().is_success() {
            bail!("Unable to get service: {}", resp.status());
        }

        let contents = resp.text().await?;

        if self.debug {
            let path = shellexpand::tilde(&format!("{}/service.json", DEBUG_DIR)).to_string();
            fs::write(path, &contents).expect("Couldn't write service.json");
        }

        self.service = match serde_json::from_str::<rtt::ServiceResponse>(&contents) {
            Ok(parsed) => Some(parsed),
            Err(err) => {
                bail!("Unable to parse service: {} {}", err, url);
            }
        };
        self.last_service_update = Some(Instant::now());

        Ok(())
    }

    pub async fn on_tick(&mut self) -> anyhow::Result<()> {
        if self.last_service_update.unwrap().elapsed() >= self.refresh_rate {
            self.refresh_service().await?;
        }

        Ok(())
    }
}
