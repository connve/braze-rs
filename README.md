# braze-rs

Unofficial Rust SDK for Braze APIs.

> Not affiliated with or endorsed by Braze, Inc.

## Workspace layout

```
braze-rs/
├── braze/                  # User-facing crate: ergonomic wrappers, auth
├── generated/
│   └── braze/              # progenitor-generated client + openapi.yaml
└── examples/
    └── braze/              # Runnable examples
```

The generated crate is regenerated from `generated/braze/openapi.yaml` at
build time. The wrapper crate (`braze`) adds typed auth, error handling, and
ergonomic per-endpoint modules on top.

## Quick start

Every API surface sits behind a Cargo feature; none are enabled by default.

```toml
[dependencies]
braze = { version = "0.2", features = ["export", "messaging"] }
```

```rust
use braze::{Client, export::ExportUsersByIdsRequest};

let client = Client::builder()
    .credentials_file(std::env::var("BRAZE_CREDENTIALS")?)?
    .build()?;

let response = client
    .export()
    .users_by_ids(&ExportUsersByIdsRequest {
        external_ids: vec!["user_1".into()],
        fields_to_export: vec!["email".into()],
        ..Default::default()
    })
    .send()
    .await?;
```

## Credentials

Credentials load from a JSON file pointed to by `BRAZE_CREDENTIALS`:

```json
{
  "api_key": "YOUR-REST-API-KEY",
  "rest_endpoint": "https://rest.iad-01.braze.com"
}
```

`rest_endpoint` is your per-instance Braze REST URL — find it in the Braze
dashboard under *Settings → API Keys*.

## Examples

```bash
export BRAZE_CREDENTIALS=$PWD/credentials.json
cargo run --example export_users --features export -- external_user_id_1 external_user_id_2
cargo run --example export_segment --features export -- <segment_id>
cargo run --example trigger_campaign --features messaging -- <campaign_id> external_user_id_1
```

`trigger_campaign` delivers real messages — point it at a test campaign.

## Coverage

| API | Feature | Endpoint | Status |
| --- | --- | --- | --- |
| Export | `export` | `POST /users/export/ids` | ✅ |
| Export | `export` | `POST /users/export/segment` | ✅ |
| Export | `export` | `POST /users/export/global_control_group` | ✅ |
| Messaging | `messaging` | `POST /messages/send` | ✅ |
| Messaging | `messaging` | `POST /campaigns/trigger/send` | ✅ |
| Messaging | `messaging` | `POST /canvas/trigger/send` | ✅ |
| Messaging | `messaging` | `POST /sends/id/create` | ✅ |
| Messaging | `messaging` | `POST /transactional/v1/campaigns/{campaign_id}/send` | ✅ |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for repo layout, spec normalisation
rules, and how to add a new endpoint.

## License

MPL-2.0
