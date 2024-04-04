use std::{error::Error as StdErr, rc::Rc};

use crate::{
    CreateRecord, CreateRecordError, CreateZone, CreateZoneError, DeleteRecord, DeleteRecordError,
    DeleteZone, DeleteZoneError, Provider, Record, RecordData, RetrieveRecordError,
    RetrieveZoneError, Zone,
};

mod api;

const SUPPORTED_RECORD_TYPES: &[&str; 14] = &[
    "A", "AAAA", "NS", "MX", "CNAME", "RP", "TXT", "SOA", "HINFO", "SRV", "DANE", "TLSA", "DS",
    "CAA",
];

#[derive(Debug)]
pub struct HetznerProvider {
    api_client: Rc<api::Client>,
}

impl Clone for HetznerProvider {
    fn clone(&self) -> Self {
        return HetznerProvider {
            api_client: Rc::from(self.api_client.as_ref().clone()),
        };
    }
}

impl HetznerProvider {
    pub fn new(api_key: &str) -> Result<Self, Box<dyn StdErr>> {
        let api_client = api::Client::new(api_key)?;
        Ok(Self {
            api_client: Rc::new(api_client),
        })
    }
}

impl Provider for HetznerProvider {
    type Zone = HetznerZone;
    type CustomRetrieveError = reqwest::Error;

    async fn get_zone(
        &self,
        zone_id: &str,
    ) -> Result<Self::Zone, RetrieveZoneError<Self::CustomRetrieveError>> {
        let response = self
            .api_client
            .retrieve_zone(zone_id)
            .await
            .map_err(|err| {
                if err.is_status() {
                    return match err.status().unwrap() {
                        reqwest::StatusCode::NOT_FOUND => RetrieveZoneError::NotFound,
                        reqwest::StatusCode::UNAUTHORIZED => RetrieveZoneError::Unauthorized,
                        _ => RetrieveZoneError::Custom(err),
                    };
                }
                RetrieveZoneError::Custom(err)
            })?;

        Ok(HetznerZone {
            api_client: self.api_client.clone(),
            repr: response.zone,
        })
    }

    async fn list_zones(
        &self,
    ) -> Result<Vec<Self::Zone>, RetrieveZoneError<Self::CustomRetrieveError>> {
        let mut zones = Vec::new();
        let mut total: Option<usize> = None;
        let mut page = 1;

        loop {
            let result =
                self.api_client
                    .retrieve_zones(page, 100)
                    .await
                    .map_err(|err| {
                        if err.is_status() {
                            return match err.status().unwrap() {
                                reqwest::StatusCode::NOT_FOUND => RetrieveZoneError::NotFound,
                                reqwest::StatusCode::UNAUTHORIZED
                                | reqwest::StatusCode::FORBIDDEN => RetrieveZoneError::Unauthorized,
                                _ => RetrieveZoneError::Custom(err),
                            };
                        }
                        RetrieveZoneError::Custom(err)
                    });

            match result {
                Ok(response) => {
                    if total.is_none() {
                        total = Some(response.meta.pagination.total_entries as usize);
                    }

                    zones.append(
                        response
                            .zones
                            .into_iter()
                            .map(|zone| HetznerZone {
                                api_client: self.api_client.clone(),
                                repr: zone,
                            })
                            .collect::<Vec<HetznerZone>>()
                            .as_mut(),
                    );
                }
                Err(err) => {
                    if let RetrieveZoneError::NotFound = err {
                        break;
                    }
                    return Err(err);
                }
            }

            if total.is_some_and(|t| zones.len() == t) {
                break;
            }

            page += 1;
        }

        Ok(zones)
    }
}

impl CreateZone for HetznerProvider {
    type CustomCreateError = reqwest::Error;

    async fn create_zone(
        &self,
        domain: &str,
    ) -> Result<Self::Zone, CreateZoneError<Self::CustomCreateError>> {
        let response = self.api_client.create_zone(domain).await.map_err(|err| {
            if err.is_status() {
                return match err.status().unwrap() {
                    reqwest::StatusCode::UNAUTHORIZED => CreateZoneError::Unauthorized,
                    reqwest::StatusCode::UNPROCESSABLE_ENTITY => CreateZoneError::InvalidDomainName,
                    _ => CreateZoneError::Custom(err),
                };
            }
            CreateZoneError::Custom(err)
        })?;

        Ok(HetznerZone {
            api_client: self.api_client.clone(),
            repr: response.zone,
        })
    }
}

impl DeleteZone for HetznerProvider {
    type CustomDeleteError = reqwest::Error;

    async fn delete_zone(
        &self,
        zone_id: &str,
    ) -> Result<(), DeleteZoneError<Self::CustomDeleteError>> {
        self.api_client.delete_zone(zone_id).await.map_err(|err| {
            if err.is_status() {
                return match err.status().unwrap() {
                    reqwest::StatusCode::NOT_FOUND => DeleteZoneError::NotFound,
                    reqwest::StatusCode::UNAUTHORIZED => DeleteZoneError::Unauthorized,
                    _ => DeleteZoneError::Custom(err),
                };
            }
            DeleteZoneError::Custom(err)
        })
    }
}

#[derive(Debug, Clone)]
pub struct HetznerZone {
    api_client: Rc<api::Client>,
    repr: api::Zone,
}

impl Zone for HetznerZone {
    type CustomRetrieveError = reqwest::Error;

    fn id(&self) -> &str {
        &self.repr.id
    }

    fn domain(&self) -> &str {
        &self.repr.name
    }

    async fn list_records(
        &self,
    ) -> Result<Vec<Record>, RetrieveRecordError<Self::CustomRetrieveError>> {
        let mut records = Vec::new();
        let mut total: Option<usize> = None;
        let mut page = 1;

        loop {
            let result = self
                .api_client
                .retrieve_records(&self.repr.id, page, 100)
                .await
                .map_err(|err| {
                    if err.is_status() {
                        return match err.status().unwrap() {
                            reqwest::StatusCode::NOT_FOUND => RetrieveRecordError::NotFound,
                            reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
                                RetrieveRecordError::Unauthorized
                            }
                            _ => RetrieveRecordError::Custom(err),
                        };
                    }
                    RetrieveRecordError::Custom(err)
                });

            match result {
                Ok(response) => {
                    if total.is_none() {
                        total = Some(response.meta.pagination.total_entries as usize);
                    }

                    records.append(
                        response
                            .records
                            .into_iter()
                            .map(|record| record.into_generic(self.repr.ttl))
                            .collect::<Vec<Record>>()
                            .as_mut(),
                    );
                }
                Err(err) => {
                    if let RetrieveRecordError::NotFound = err {
                        break;
                    }
                    return Err(err);
                }
            }

            if total.is_some_and(|t| records.len() == t) {
                break;
            }

            page += 1;
        }

        Ok(records)
    }

    async fn get_record(
        &self,
        record_id: &str,
    ) -> Result<Record, RetrieveRecordError<Self::CustomRetrieveError>> {
        let response = self
            .api_client
            .retrieve_record(record_id)
            .await
            .map_err(|err| {
                if err.is_status() {
                    return match err.status().unwrap() {
                        reqwest::StatusCode::NOT_FOUND => RetrieveRecordError::NotFound,
                        reqwest::StatusCode::UNAUTHORIZED => RetrieveRecordError::Unauthorized,
                        _ => RetrieveRecordError::Custom(err),
                    };
                }
                RetrieveRecordError::Custom(err)
            })?;

        if response.record.zone_id != self.repr.id {
            return Err(RetrieveRecordError::NotFound);
        }

        Ok(response.record.into_generic(self.repr.ttl))
    }
}

impl CreateRecord for HetznerZone {
    type CustomCreateError = reqwest::Error;

    async fn create_record(
        &self,
        host: &str,
        data: &RecordData,
        ttl: u64,
    ) -> Result<Record, CreateRecordError<Self::CustomCreateError>> {
        let typ = data.get_type();
        if !SUPPORTED_RECORD_TYPES.iter().any(|r| *r == typ) {
            return Err(CreateRecordError::UnsupportedType);
        }

        let mut opt_ttl = None;
        if ttl != self.repr.ttl {
            opt_ttl = Some(ttl);
        }

        let response = self
            .api_client
            .create_record(
                &self.repr.id,
                host,
                data.get_type(),
                data.get_value().as_str(),
                opt_ttl,
            )
            .await
            .map_err(|err| {
                if err.is_status() {
                    return match err.status().unwrap() {
                        reqwest::StatusCode::UNAUTHORIZED => CreateRecordError::Unauthorized,
                        reqwest::StatusCode::UNPROCESSABLE_ENTITY => {
                            CreateRecordError::InvalidRecord
                        }
                        _ => CreateRecordError::Custom(err),
                    };
                }
                CreateRecordError::Custom(err)
            })?;

        Ok(response.record.into_generic(self.repr.ttl))
    }
}

impl DeleteRecord for HetznerZone {
    type CustomDeleteError = reqwest::Error;

    async fn delete_record(
        &self,
        record_id: &str,
    ) -> Result<(), DeleteRecordError<Self::CustomDeleteError>> {
        self.get_record(record_id).await.map_err(|err| match err {
            RetrieveRecordError::Unauthorized => DeleteRecordError::Unauthorized,
            RetrieveRecordError::NotFound => DeleteRecordError::NotFound,
            RetrieveRecordError::Custom(rerr) => DeleteRecordError::Custom(rerr),
        })?;

        self.api_client
            .delete_record(record_id)
            .await
            .map_err(|err| {
                if err.is_status() {
                    return match err.status().unwrap() {
                        reqwest::StatusCode::NOT_FOUND => DeleteRecordError::NotFound,
                        reqwest::StatusCode::UNAUTHORIZED => DeleteRecordError::Unauthorized,
                        _ => DeleteRecordError::Custom(err),
                    };
                }
                DeleteRecordError::Custom(err)
            })
    }
}

impl api::Record {
    pub fn into_generic(self, default_ttl: u64) -> Record {
        Record {
            id: self.id,
            host: self.name,
            data: RecordData::from_raw(self.typ.as_str(), self.value.as_str()),
            ttl: self.ttl.unwrap_or(default_ttl),
        }
    }
}
