# libdns-rs

[![crates.io](https://img.shields.io/crates/v/libdns.svg)](https://crates.io/crates/libdns)
[![Docs](https://docs.rs/libdns/badge.svg)](https://docs.rs/libdns)
![Build and check](https://github.com/lus/libdns-rs/actions/workflows/build_and_check.yml/badge.svg)

This project is a rip-off of [libdns](https://github.com/libdns/libdns) written in Rust.
It defines an abstract API for managing DNS zones and implements it for several widely-used providers.

> [!NOTE]
> This project is my very first (serious) attempt at learning Rust. I am more than thankful for any suggestions and tips on this matter, so please feel welcomed to bring them up in an issue :)

## Using

To add `libdns` to your project, an entry like the following would be enough to include only the abstract DNS zone management traits:

```toml
[dependencies]
libdns = { version = "0" }
```

### Including provider implementations

If you need one or more concrete provider implementations as well, you can simply add their corresponding feature flags to the dependency's `features` field:

| Provider                                        | Feature Flag |
|-------------------------------------------------|--------------|
| [Hetzner](https://www.hetzner.com/dns-console/) | `hetzner`    |

### Choosing TLS backend

The provider implementations use [`reqwest`](https://crates.io/crates/reqwest) for communicating with their APIs whenever possible.
By default, the `default-tls` feature is enabled for reqwest.
These features can be given instead for choosing a different TLS backend (remember to disable the default features):

- `default-tls` (default)
- `rustls-tls`
- `native-tls`
- `native-tls-vendor`

Please refer to [`reqwest`s docs](https://docs.rs/reqwest/0.12.2/reqwest/#optional-features) for an overview on what TLS backend does what.

## Contributing

I am grateful for any contribution to this project, so feel free to request, add or fix provider implementations when neccessary.
