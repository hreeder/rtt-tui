use anyhow::bail;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::{fs::{File, self}, time::{Duration, Instant}};

use crate::rtt;
use rtt::StringOrStringVec;

#[derive(Deserialize)]
struct ConfigFile {
    username: String,
    password: String,
}

const API_BASE: &str = "https://api.rtt.io/api/v1/json";

pub(crate) struct App {
    destination: Option<rtt::Location>,
    location_service: Option<rtt::LocationService>,
    pub(crate) service: Option<rtt::ServiceResponse>,

    pub(crate) should_quit: bool,
    pub(crate) last_service_update: Option<Instant>,
    refresh_rate: Duration,

    config: ConfigFile,
    http: reqwest::Client,
}

impl App {
    pub fn new(refresh_rate: Duration) -> App{
        let cfg_path = shellexpand::tilde("~/.config/rtt.yaml").to_string();
        let cfg_file = File::open(cfg_path).unwrap();
        let config: ConfigFile = serde_yaml::from_reader(&cfg_file).unwrap();

        App {
            destination: None,
            location_service: None,
            service: None,

            should_quit: false,
            last_service_update: None,
            refresh_rate,

            config,
            http: reqwest::Client::new(),
        }
    }

    pub fn on_key(&mut self, c: char) {
        // We're allowing single_match here because we're assuming that we'll likely
        // want to support other keys in future
        #[allow(clippy::single_match)]
        match c {
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
        
        self.location_service = match filtered_services.first() {
            Some(service) => Some(service.clone()),
            None => {
                bail!("Unable to find service");
            }
        };

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

        self.service = match resp.json::<rtt::ServiceResponse>().await {
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
