use std::{error::Error as StdErr, rc::Rc};

use crate::{Provider, Record, RecordData, RetrieveRecordError, RetrieveZoneError, Zone};

mod api;

pub struct HetznerProvider {
    api_client: Rc<api::Client>,
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
    type CustomRetrieveError = ();

    fn get_zone(
        &self,
        zone_id: &str,
    ) -> Result<Self::Zone, RetrieveZoneError<Self::CustomRetrieveError>> {
        let zone = self.api_client.retrieve_zone(zone_id).map_err(|err| {
            if err.is_status() {
                return match err.status().unwrap() {
                    reqwest::StatusCode::NOT_FOUND => RetrieveZoneError::NotFound,
                    reqwest::StatusCode::UNAUTHORIZED => RetrieveZoneError::Unauthorized,
                    _ => RetrieveZoneError::Custom(()),
                };
            }
            RetrieveZoneError::Custom(())
        })?;

        Ok(HetznerZone {
            api_client: self.api_client.clone(),
            repr: zone.zone,
        })
    }

    fn list_zones(&self) -> Result<Vec<Self::Zone>, RetrieveZoneError<Self::CustomRetrieveError>> {
        let mut zones = Vec::new();
        let mut total: Option<usize> = None;
        let mut page = 1;

        loop {
            let new = self.api_client.retrieve_zones(page, 100).map_err(|err| {
                if err.is_status() {
                    return match err.status().unwrap() {
                        reqwest::StatusCode::NOT_FOUND => RetrieveZoneError::NotFound,
                        reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
                            RetrieveZoneError::Unauthorized
                        }
                        _ => RetrieveZoneError::Custom(()),
                    };
                }
                println!("{:?}", err);
                RetrieveZoneError::Custom(())
            });

            match new {
                Ok(v) => {
                    if total == None {
                        total = Some(v.meta.pagination.total_entries as usize);
                    }

                    zones.append(
                        v.zones
                            .into_iter()
                            .map(|z| HetznerZone {
                                api_client: self.api_client.clone(),
                                repr: z,
                            })
                            .collect::<Vec<HetznerZone>>()
                            .as_mut(),
                    );
                }
                Err(err) => {
                    if err == RetrieveZoneError::NotFound {
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

pub struct HetznerZone {
    api_client: Rc<api::Client>,
    repr: api::Zone,
}

impl Zone for HetznerZone {
    type CustomRetrieveError = ();

    fn id(&self) -> &str {
        &self.repr.id
    }

    fn domain(&self) -> &str {
        &self.repr.name
    }

    fn list_records(&self) -> Result<Vec<Record>, RetrieveRecordError<Self::CustomRetrieveError>> {
        let mut records = Vec::new();
        let mut total: Option<usize> = None;
        let mut page = 1;

        loop {
            let new = self
                .api_client
                .retrieve_records(&self.repr.id, page, 100)
                .map_err(|err| {
                    if err.is_status() {
                        return match err.status().unwrap() {
                            reqwest::StatusCode::NOT_FOUND => RetrieveRecordError::NotFound,
                            reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
                                RetrieveRecordError::Unauthorized
                            }
                            _ => RetrieveRecordError::Custom(()),
                        };
                    }
                    RetrieveRecordError::Custom(())
                });

            match new {
                Ok(v) => {
                    if total == None {
                        total = Some(v.meta.pagination.total_entries as usize);
                    }

                    records.append(
                        v.records
                            .into_iter()
                            .map(|r| r.into_generic(self.repr.ttl))
                            .collect::<Vec<Record>>()
                            .as_mut(),
                    );
                }
                Err(err) => {
                    if err == RetrieveRecordError::NotFound {
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

    fn get_record(
        &self,
        record_id: &str,
    ) -> Result<crate::Record, RetrieveRecordError<Self::CustomRetrieveError>> {
        let record = self.api_client.retrieve_record(record_id).map_err(|err| {
            if err.is_status() {
                return match err.status().unwrap() {
                    reqwest::StatusCode::NOT_FOUND => RetrieveRecordError::NotFound,
                    reqwest::StatusCode::UNAUTHORIZED => RetrieveRecordError::Unauthorized,
                    _ => RetrieveRecordError::Custom(()),
                };
            }
            RetrieveRecordError::Custom(())
        })?;

        if record.record.zone_id != self.repr.id {
            return Err(RetrieveRecordError::NotFound);
        }

        Ok(record.record.into_generic(self.repr.ttl))
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
