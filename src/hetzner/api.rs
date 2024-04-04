use std::{borrow::Cow, collections::HashMap, error::Error};

use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client as HttpClient,
};
use serde::Deserialize;

const HETZNER_API_URL: &str = "https://dns.hetzner.com/api/v1";

#[derive(Debug, Clone)]
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

    pub async fn retrieve_zones(
        &self,
        page: u32,
        per_page: u32,
    ) -> Result<ZonesResponse, reqwest::Error> {
        self.http_client
            .get(format!(
                "{}/zones?page={}&per_page={}",
                HETZNER_API_URL, page, per_page
            ))
            .send()
            .await?
            .json::<ZonesResponse>()
            .await
    }

    pub async fn retrieve_zone(&self, zone_id: &str) -> Result<ZoneResponse, reqwest::Error> {
        self.http_client
            .get(format!("{}/zones/{}", HETZNER_API_URL, zone_id))
            .send()
            .await?
            .json()
            .await
    }

    pub async fn create_zone(&self, domain: &str) -> Result<ZoneResponse, reqwest::Error> {
        let mut request_body = HashMap::new();
        request_body.insert("name", domain);

        self.http_client
            .post(format!("{}/zones", HETZNER_API_URL))
            .json(&request_body)
            .send()
            .await?
            .json()
            .await
    }

    pub async fn delete_zone(&self, zone_id: &str) -> Result<(), reqwest::Error> {
        self.http_client
            .delete(format!("{}/zones/{}", HETZNER_API_URL, zone_id))
            .send()
            .await
            .map(|_| ())
    }

    pub async fn retrieve_records(
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
            .send()
            .await?
            .json()
            .await
    }

    pub async fn retrieve_record(&self, record_id: &str) -> Result<RecordResponse, reqwest::Error> {
        self.http_client
            .get(format!("{}/records/{}", HETZNER_API_URL, record_id))
            .send()
            .await?
            .json()
            .await
    }

    pub async fn create_record(
        &self,
        zone_id: &str,
        host: &str,
        typ: &str,
        value: &str,
        ttl: Option<u64>,
    ) -> Result<RecordResponse, reqwest::Error> {
        let mut request_body = HashMap::from([
            ("zone_id", Cow::Borrowed(zone_id)),
            ("name", Cow::Borrowed(host)),
            ("type", Cow::Borrowed(typ)),
            ("value", Cow::Borrowed(value)),
        ]);

        if let Some(ttl_str) = ttl.map(|r| r.to_string()) {
            request_body.insert("ttl", Cow::Owned(ttl_str.to_string()));
        }

        self.http_client
            .post(format!("{}/records", HETZNER_API_URL))
            .json(&request_body)
            .send()
            .await?
            .json()
            .await
    }

    pub async fn delete_record(&self, record_id: &str) -> Result<(), reqwest::Error> {
        self.http_client
            .delete(format!("{}/records/{}", HETZNER_API_URL, record_id))
            .send()
            .await
            .map(|_| ())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Deserialize)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub status: ZoneStatus,
    pub ttl: u64,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ZoneStatus {
    Verified,
    Failed,
    Pending,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Deserialize)]
pub struct ZoneResponse {
    pub zone: Zone,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Deserialize)]
pub struct ZonesResponse {
    pub meta: Meta,
    pub zones: Vec<Zone>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Deserialize)]
pub struct Record {
    pub id: String,
    pub name: String,
    pub ttl: Option<u64>,
    #[serde(rename = "type")]
    pub typ: String,
    pub value: String,
    pub zone_id: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Deserialize)]
pub struct RecordResponse {
    pub record: Record,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Deserialize)]
pub struct RecordsResponse {
    pub meta: Meta,
    pub records: Vec<Record>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Deserialize)]
pub struct Meta {
    pub pagination: Pagination,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Deserialize)]
pub struct Pagination {
    pub last_page: u32,
    pub page: u32,
    pub per_page: u32,
    pub total_entries: u32,
}
