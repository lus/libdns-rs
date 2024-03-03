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

use std::net::{Ipv4Addr, Ipv6Addr};

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
    type CustomRetrieveError;

    /// Retrieves all available zones.
    /// <br/>When no record exists, an [`Ok`] value with an empty [`Vec`] will be returned, not [`RetrieveZoneError::NotFound`].
    fn list_zones(&self) -> Result<Vec<&Self::Zone>, RetrieveZoneError<Self::CustomRetrieveError>>;

    /// Retrieves a zone by its provider-specific ID.
    /// <br/>Refer to the provider's documentation to figure out which value is used as the ID.
    fn get_zone(
        &self,
        zone_id: &str,
    ) -> Result<&Self::Zone, RetrieveZoneError<Self::CustomRetrieveError>>;
}

/// Represents an error that occured when retrieving DNS zones using [`Provider::list_zones`] or [`Provider::get_zone`].
///
/// Providers can provide a custom error type ([`Provider::CustomRetrieveError`]) and return it using [`RetrieveZoneError::Custom`] to extend the pool of well-defined errors.
/// <br/>Refer to the provider's documentation for more information.
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
    type CustomCreateError;

    /// Creates a new DNS zone with the given domain.
    fn create_zone(
        &self,
        domain: &str,
    ) -> Result<&Self::Zone, CreateZoneError<Self::CustomCreateError>>;
}

/// Represents an error that occured when creating DNS zones using [`CreateZone::create_zone`].
///
/// Providers can provide a custom error type ([`CreateZone::CustomCreateError`]) and return it using [`CreateZoneError::Custom`] to extend the pool of well-defined errors.
/// <br/>Refer to the provider's documentation for more information.
pub enum CreateZoneError<T> {
    /// Indicates that the DNS provider is not authorized to execute this action.
    Unauthorized,

    /// Provides a custom, provider-specific error of type `T`.
    Custom(T),
}

/// Represents a [`Provider`] that supports zone deletion.
pub trait DeleteZone: Provider {
    /// The provider-specific custom zone deletion error type used for [`DeleteZoneError::Custom`].
    /// <br/>If no custom errors should be provided, use `()`.
    type CustomDeleteError;

    /// Deletes a zone by its provider-specific ID.
    /// <br/>Refer to the provider's documentation to figure out which value is used as the ID.
    fn delete_zone(&self, zone_id: &str) -> Result<(), DeleteZoneError<Self::CustomDeleteError>>;
}

/// Represents an error that occured when deleting DNS zones using [`DeleteZone::delete_zone`].
///
/// Providers can provide a custom error type ([`DeleteZone::CustomDeleteError`]) and return it using [`DeleteZoneError::Custom`] to extend the pool of well-defined errors.
/// <br/>Refer to the provider's documentation for more information.
pub enum DeleteZoneError<T> {
    /// Indicates that the DNS provider is not authorized to execute this action.
    Unauthorized,

    /// Indicates that there is no zone with the given ID.
    NotFound,

    /// Provides a custom, provider-specific error of type `T`.
    Custom(T),
}

/// Represents a DNS record value.
pub enum RecordData<'a> {
    A(Ipv4Addr),
    AAAA(Ipv6Addr),
    CNAME(&'a str),
    MX {
        priority: u16,
        mail_server: &'a str,
    },
    NS(&'a str),
    SRV {
        priority: u16,
        weight: u16,
        port: u16,
        target: &'a str,
    },
    TXT(&'a str),
}

/// Represents a DNS record.
pub struct Record<'a> {
    pub id: &'a str,
    pub host: &'a str,
    pub data: &'a RecordData<'a>,
    pub ttl: u32,
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
    type CustomRetrieveError;

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
    type CustomCreateError;

    /// Creates a new record.
    fn create_record(
        &self,
        host: &str,
        data: &RecordData,
        ttl: u32,
    ) -> Result<Record, CreateRecordError<Self::CustomCreateError>>;
}

/// Represents an error that occured when creating DNS records using [`CreateRecord::create_record`].
///
/// Providers can provide a custom error type ([`CreateRecord::CustomCreateError`]) and return it using [`CreateRecordError::Custom`] to extend the pool of well-defined errors.
/// <br/>Refer to the provider's documentation for more information.
pub enum CreateRecordError<T> {
    /// Indicates that the DNS provider is not authorized to execute this action.
    Unauthorized,

    /// Indicates that the DNS provider does not support the specified record type.
    UnsupportedType,

    /// Provides a custom, provider-specific error of type `T`.
    Custom(T),
}

/// Represents a [`Zone`] that supports record deletion.
pub trait DeleteRecord: Zone {
    /// The provider-specific custom record creation error type used for [`DeleteRecordError::Custom`].
    /// <br/>If no custom errors should be provided, use `()`.
    type CustomDeleteError;

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
pub enum DeleteRecordError<T> {
    /// Indicates that the DNS provider is not authorized to execute this action.
    Unauthorized,

    /// Indicates that there is no record with the given ID.
    NotFound,

    /// Provides a custom, provider-specific error of type `T`.
    Custom(T),
}
