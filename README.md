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
braze = { version = "0.4", features = ["export", "messaging"] }
```

```rust
let client = braze::Client::builder()
    .api_key(api_key)
    .base_url("https://rest.iad-01.braze.com")
    .build()?;
```

## Coverage

| API | Feature | Methods |
| --- | --- | --- |
| Export | `export` | [`users_by_ids`](https://www.braze.com/docs/api/endpoints/export/user_data/post_users_identifier/), [`users_by_segment`](https://www.braze.com/docs/api/endpoints/export/user_data/post_users_segment/), [`users_by_global_control_group`](https://www.braze.com/docs/api/endpoints/export/user_data/post_users_global_control_group/) |
| Messaging | `messaging` | [`send_messages`](https://www.braze.com/docs/api/endpoints/messaging/send_messages/post_send_messages/), [`send_triggered_campaigns`](https://www.braze.com/docs/api/endpoints/messaging/send_messages/post_send_triggered_campaigns/), [`send_triggered_canvases`](https://www.braze.com/docs/api/endpoints/messaging/send_messages/post_send_triggered_canvases/), [`create_send_ids`](https://www.braze.com/docs/api/endpoints/messaging/send_messages/post_create_send_ids/), [`send_transactional_message`](https://www.braze.com/docs/api/endpoints/messaging/send_messages/post_send_transactional_message/) |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MPL-2.0
