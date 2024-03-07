//! Abstracting and implementing DNS zone management for different providers.
//!
//! This crate defines a generic provider-agnostic API to manage DNS zones and optionally provides implementations for well-known providers.
//!
//! # Providers
//!
//! The most basic trait for every DNS zone provider is [`Provider`]. It only support zone retrieval by default.
//! <br/>The following capabilities can be implemented additionally:
//!
//! - [`CreateZone`]
//! - [`DeleteZone`]
//!
//! # Zones
//!
//! The generic DNS [`Zone`] also only supports record retrieval by default.
//! <br/>The following capabilities can be implemented additionally:
//!
//! - [`CreateRecord`]
//! - [`DeleteRecord`]

use std::{
    fmt::Debug,
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

#[cfg(feature = "hetzner")]
pub mod hetzner;

/// Represents a DNS zone provider.
///
/// Providers implement [`Zone`] management, which in turn implement [`Record`] management.
/// By default, only zone retrieval is supported, but the following additional capabilities may be implemented to allow further zone management:
///
/// - [`CreateZone`]
/// - [`DeleteZone`]
pub trait Provider {
    /// The provider-specific zone type.
    type Zone: Zone;

    /// The provider-specific custom zone retrieval error type used for [`RetrieveZoneError::Custom`].
    /// <br/>If no custom errors should be provided, use `()`.
    type CustomRetrieveError: Debug;

    /// Retrieves all available zones.
    /// <br/>When no record exists, an [`Ok`] value with an empty [`Vec`] will be returned, not [`RetrieveZoneError::NotFound`].
    fn list_zones(&self) -> Result<Vec<Self::Zone>, RetrieveZoneError<Self::CustomRetrieveError>>;

    /// Retrieves a zone by its provider-specific ID.
    /// <br/>Refer to the provider's documentation to figure out which value is used as the ID.
    fn get_zone(
        &self,
        zone_id: &str,
    ) -> Result<Self::Zone, RetrieveZoneError<Self::CustomRetrieveError>>;
}

/// Represents an error that occured when retrieving DNS zones using [`Provider::list_zones`] or [`Provider::get_zone`].
///
/// Providers can provide a custom error type ([`Provider::CustomRetrieveError`]) and return it using [`RetrieveZoneError::Custom`] to extend the pool of well-defined errors.
/// <br/>Refer to the provider's documentation for more information.
#[derive(Debug, PartialEq)]
pub enum RetrieveZoneError<T> {
    /// Indicates that the DNS provider is not authorized to execute this action.
    Unauthorized,

    /// Indicates that there is no zone with the given ID.
    NotFound,

    /// Provides a custom, provider-specific error of type `T`.
    Custom(T),
}

/// Represents a [`Provider`] that supports zone creation.
pub trait CreateZone: Provider {
    /// The provider-specific custom zone creation error type used for [`CreateZoneError::Custom`].
    /// <br/>If no custom errors should be provided, use `()`.
    type CustomCreateError: Debug;

    /// Creates a new DNS zone with the given domain.
    fn create_zone(
        &self,
        domain: &str,
    ) -> Result<Self::Zone, CreateZoneError<Self::CustomCreateError>>;
}

/// Represents an error that occured when creating DNS zones using [`CreateZone::create_zone`].
///
/// Providers can provide a custom error type ([`CreateZone::CustomCreateError`]) and return it using [`CreateZoneError::Custom`] to extend the pool of well-defined errors.
/// <br/>Refer to the provider's documentation for more information.
#[derive(Debug, PartialEq)]
pub enum CreateZoneError<T> {
    /// Indicates that the DNS provider is not authorized to execute this action.
    Unauthorized,

    /// Indicates that the specified domain name was not accepted.
    InvalidDomainName,

    /// Provides a custom, provider-specific error of type `T`.
    Custom(T),
}

/// Represents a [`Provider`] that supports zone deletion.
pub trait DeleteZone: Provider {
    /// The provider-specific custom zone deletion error type used for [`DeleteZoneError::Custom`].
    /// <br/>If no custom errors should be provided, use `()`.
    type CustomDeleteError: Debug;

    /// Deletes a zone by its provider-specific ID.
    /// <br/>Refer to the provider's documentation to figure out which value is used as the ID.
    fn delete_zone(&self, zone_id: &str) -> Result<(), DeleteZoneError<Self::CustomDeleteError>>;
}

/// Represents an error that occured when deleting DNS zones using [`DeleteZone::delete_zone`].
///
/// Providers can provide a custom error type ([`DeleteZone::CustomDeleteError`]) and return it using [`DeleteZoneError::Custom`] to extend the pool of well-defined errors.
/// <br/>Refer to the provider's documentation for more information.
#[derive(Debug, PartialEq)]
pub enum DeleteZoneError<T> {
    /// Indicates that the DNS provider is not authorized to execute this action.
    Unauthorized,

    /// Indicates that there is no zone with the given ID.
    NotFound,

    /// Provides a custom, provider-specific error of type `T`.
    Custom(T),
}

/// Represents a DNS record value.
#[derive(Debug)]
pub enum RecordData {
    A(Ipv4Addr),
    AAAA(Ipv6Addr),
    CNAME(String),
    MX {
        priority: u16,
        mail_server: String,
    },
    NS(String),
    SRV {
        priority: u16,
        weight: u16,
        port: u16,
        target: String,
    },
    TXT(String),
    Other {
        typ: String,
        value: String,
    },
}

impl RecordData {
    /// Tries to parse raw DNS record data to their corresponsing [`RecordData`] value.
    ///
    /// This function falls back to [`RecordData::Other`] if the value could not be parsed or the type is not supported.
    pub fn from_raw(typ: &str, value: &str) -> RecordData {
        let data = match typ {
            "A" => Ipv4Addr::from_str(value)
                .ok()
                .map(|addr| RecordData::A(addr)),
            "AAAA" => Ipv6Addr::from_str(value)
                .ok()
                .map(|addr| RecordData::AAAA(addr)),
            "CNAME" => Some(RecordData::CNAME(value.to_owned())),
            "MX" => {
                let mut iter = value.split_whitespace();

                let priority = iter.next().and_then(|raw| raw.parse::<u16>().ok());
                let server = iter.next();

                if priority.is_none() || server.is_none() {
                    None
                } else {
                    Some(RecordData::MX {
                        priority: priority.unwrap(),
                        mail_server: server.unwrap().to_owned(),
                    })
                }
            }
            "NS" => Some(RecordData::NS(value.to_owned())),
            "SRV" => {
                let mut iter = value.split_whitespace();

                let priority = iter.next().and_then(|raw| raw.parse::<u16>().ok());
                let weight = iter.next().and_then(|raw| raw.parse::<u16>().ok());
                let port = iter.next().and_then(|raw| raw.parse::<u16>().ok());
                let target = iter.next();

                if priority.is_none() || weight.is_none() || port.is_none() || target.is_none() {
                    None
                } else {
                    Some(RecordData::SRV {
                        priority: priority.unwrap(),
                        weight: weight.unwrap(),
                        port: port.unwrap(),
                        target: target.unwrap().to_owned(),
                    })
                }
            }
            "TXT" => Some(RecordData::TXT(value.to_owned())),
            _ => None,
        };

        data.unwrap_or(RecordData::Other {
            typ: typ.to_owned(),
            value: value.to_owned(),
        })
    }

    pub fn get_type(&self) -> &str {
        match self {
            RecordData::A(_) => "A",
            RecordData::AAAA(_) => "A",
            RecordData::CNAME(_) => "CNAME",
            RecordData::MX { .. } => "MX",
            RecordData::NS(_) => "NS",
            RecordData::SRV { .. } => "SRV",
            RecordData::TXT(_) => "TXT",
            RecordData::Other { typ, .. } => typ.as_str(),
        }
    }

    pub fn get_value(&self) -> String {
        match self {
            RecordData::A(addr) => addr.to_string(),
            RecordData::AAAA(addr) => addr.to_string(),
            RecordData::CNAME(alias) => alias.clone(),
            RecordData::MX {
                priority,
                mail_server,
            } => format!("{} {}", priority, mail_server),
            RecordData::NS(ns) => ns.clone(),
            RecordData::SRV {
                priority,
                weight,
                port,
                target,
            } => format!("{} {} {} {}", priority, weight, port, target),
            RecordData::TXT(val) => val.clone(),
            RecordData::Other { value, .. } => value.clone(),
        }
    }
}

/// Represents a DNS record.
#[derive(Debug)]
pub struct Record {
    pub id: String,
    pub host: String,
    pub data: RecordData,
    pub ttl: u64,
}

/// Represents a DNS zone.
///
/// DNS zones are provided by a DNS [`Provider`] and implement [`Record`] management.
/// By default, only record retrieval is supported, but the following capabilities may be implemented to allow further record management:
///
/// - [`CreateRecord`]
/// - [`CreateRecord`]
pub trait Zone {
    /// The provider-specific custom record retrieval error type used for [`RetrieveRecordError::Custom`].
    /// <br/>If no custom errors should be provided, use `()`.
    type CustomRetrieveError: Debug;

    /// Returns the provider-specific ID of the zone.
    fn id(&self) -> &str;

    /// Returns the domain the zone manages.
    fn domain(&self) -> &str;

    /// Retrieves all available records.
    /// <br/>When no record exists, an [`Ok`] value with an empty [`Vec`] will be returned, not [`RetrieveRecordError::NotFound`].
    fn list_records(&self) -> Result<Vec<Record>, RetrieveRecordError<Self::CustomRetrieveError>>;

    /// Retrieves a record by its provider-specific ID.
    /// <br/>Refer to the provider's documentation to figure out which value is used as the ID.
    fn get_record(
        &self,
        record_id: &str,
    ) -> Result<Record, RetrieveRecordError<Self::CustomRetrieveError>>;
}

/// Represents an error that occured when retrieving DNS records using [`Zone::list_records`] or [`Zone::get_record`].
///
/// Providers can provide a custom error type ([`Zone::CustomRetrieveError`]) and return it using [`RetrieveRecordError::Custom`] to extend the pool of well-defined errors.
/// <br/>Refer to the provider's documentation for more information.
#[derive(Debug, PartialEq)]
pub enum RetrieveRecordError<T> {
    /// Indicates that the DNS provider is not authorized to execute this action.
    Unauthorized,

    /// Indicates that there is no record with the given ID.
    NotFound,

    /// Provides a custom, provider-specific error of type `T`.
    Custom(T),
}

/// Represents a [`Zone`] that supports record creation.
pub trait CreateRecord: Zone {
    /// The provider-specific custom record creation error type used for [`CreateRecordError::Custom`].
    /// <br/>If no custom errors should be provided, use `()`.
    type CustomCreateError: Debug;

    /// Creates a new record.
    fn create_record(
        &self,
        host: &str,
        data: &RecordData,
        ttl: u64,
    ) -> Result<Record, CreateRecordError<Self::CustomCreateError>>;
}

/// Represents an error that occured when creating DNS records using [`CreateRecord::create_record`].
///
/// Providers can provide a custom error type ([`CreateRecord::CustomCreateError`]) and return it using [`CreateRecordError::Custom`] to extend the pool of well-defined errors.
/// <br/>Refer to the provider's documentation for more information.
#[derive(Debug, PartialEq)]
pub enum CreateRecordError<T> {
    /// Indicates that the DNS provider is not authorized to execute this action.
    Unauthorized,

    /// Indicates that the DNS provider does not support the specified record type.
    UnsupportedType,

    /// Indicates that the record value is invalid.
    InvalidRecord,

    /// Provides a custom, provider-specific error of type `T`.
    Custom(T),
}

/// Represents a [`Zone`] that supports record deletion.
pub trait DeleteRecord: Zone {
    /// The provider-specific custom record creation error type used for [`DeleteRecordError::Custom`].
    /// <br/>If no custom errors should be provided, use `()`.
    type CustomDeleteError: Debug;

    /// Deletes a record by its ID.
    fn delete_record(
        &self,
        record_id: &str,
    ) -> Result<(), DeleteRecordError<Self::CustomDeleteError>>;
}

/// Represents an error that occured when deleting DNS records using [`DeleteRecord::delete_record`].
///
/// Providers can provide a custom error type ([`DeleteRecord::CustomDeleteError`]) and return it using [`DeleteRecordError::Custom`] to extend the pool of well-defined errors.
/// <br/>Refer to the provider's documentation for more information.
#[derive(Debug, PartialEq)]
pub enum DeleteRecordError<T> {
    /// Indicates that the DNS provider is not authorized to execute this action.
    Unauthorized,

    /// Indicates that there is no record with the given ID.
    NotFound,

    /// Provides a custom, provider-specific error of type `T`.
    Custom(T),
}
