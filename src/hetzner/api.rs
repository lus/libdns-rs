use std::error::Error;

use reqwest::{
    blocking::Client as HttpClient,
    header::{HeaderMap, HeaderValue},
};
use serde::Deserialize;

const HETZNER_API_URL: &str = "https://dns.hetzner.com/api/v1/";

pub struct Client {
    http_client: HttpClient,
}

impl Client {
    pub fn new(api_key: &str) -> Result<Self, Box<dyn Error>> {
        let mut headers = HeaderMap::new();
        let mut auth_value = HeaderValue::from_str(api_key)?;
        auth_value.set_sensitive(true);
        headers.append("Auth-API-Token", auth_value);

        let http_client = HttpClient::builder().default_headers(headers).build()?;
        Ok(Self { http_client })
    }

    pub fn retrieve_zones(
        &self,
        page: u32,
        per_page: u32,
    ) -> Result<ZonesResponse, reqwest::Error> {
        self.http_client
            .get(format!(
                "{}/zones?page={}&per_page={}",
                HETZNER_API_URL, page, per_page
            ))
            .send()?
            .json()
    }

    pub fn retrieve_zone(&self, zone_id: &str) -> Result<ZoneResponse, reqwest::Error> {
        self.http_client
            .get(format!("{}/zones/{}", HETZNER_API_URL, zone_id))
            .send()?
            .json()
    }

    pub fn retrieve_records(
        &self,
        zone_id: &str,
        page: u32,
        per_page: u32,
    ) -> Result<RecordsResponse, reqwest::Error> {
        self.http_client
            .get(format!(
                "{}/records?zone_id={}&page={}&per_page={}",
                HETZNER_API_URL, zone_id, page, per_page
            ))
            .send()?
            .json()
    }

    pub fn retrieve_record(&self, record_id: &str) -> Result<RecordResponse, reqwest::Error> {
        self.http_client
            .get(format!("{}/records/{}", HETZNER_API_URL, record_id))
            .send()?
            .json()
    }
}

#[derive(Deserialize, Debug)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub status: ZoneStatus,
    pub ttl: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ZoneStatus {
    Verified,
    Failed,
    Pending,
}

#[derive(Deserialize, Debug)]
pub struct ZoneResponse {
    pub zone: Zone,
}

#[derive(Deserialize, Debug)]
pub struct ZonesResponse {
    pub meta: Meta,
    pub zones: Vec<Zone>,
}

#[derive(Deserialize, Debug)]
pub struct Record {
    pub id: String,
    pub name: String,
    pub ttl: Option<u64>,
    #[serde(rename = "type")]
    pub typ: String,
    pub value: String,
    pub zone_id: String,
}

#[derive(Deserialize, Debug)]
pub struct RecordResponse {
    pub record: Record,
}

#[derive(Deserialize, Debug)]
pub struct RecordsResponse {
    pub meta: Meta,
    pub records: Vec<Record>,
}

#[derive(Deserialize, Debug)]
pub struct Meta {
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct Pagination {
    pub last_page: u32,
    pub page: u32,
    pub per_page: u32,
    pub total_entries: u32,
}
