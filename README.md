# braze-rs

[![Test Suite](https://github.com/connve/braze-rs/actions/workflows/test.yml/badge.svg)](https://github.com/connve/braze-rs/actions/workflows/test.yml)
[![Security Audit](https://github.com/connve/braze-rs/actions/workflows/security.yml/badge.svg)](https://github.com/connve/braze-rs/actions/workflows/security.yml)
[![Release](https://img.shields.io/github/v/release/connve/braze-rs)](https://github.com/connve/braze-rs/releases)
[![Crates.io](https://img.shields.io/crates/v/braze.svg)](https://crates.io/crates/braze)
[![Docs.rs](https://docs.rs/braze/badge.svg)](https://docs.rs/braze)
[![License: MPL 2.0](https://img.shields.io/badge/License-MPL%202.0-brightgreen.svg)](https://opensource.org/licenses/MPL-2.0)

Unofficial Rust SDK for Braze APIs.

> Not affiliated with or endorsed by Braze, Inc.

## Quick start

```toml
[dependencies]
braze = { version = "0.3", features = ["export"] }
```

```rust
let client = braze::Client::builder()
    .api_key(api_key)
    .base_url("https://rest.iad-01.braze.com")
    .build()?;
```

## Coverage

| API | Feature | Endpoints |
| --- | --- | --- |
| Export | `export` | `POST /users/export/ids`, `POST /users/export/segment`, `POST /users/export/global_control_group` |
| Messaging | `messaging` | `POST /messages/send`, `POST /campaigns/trigger/send`, `POST /canvas/trigger/send`, `POST /sends/id/create` |
| Messaging | `messaging` | `POST /transactional/v1/campaigns/{campaign_id}/send` |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MPL-2.0
