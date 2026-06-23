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

```rust
use braze::{Client, Credentials, export::ExportUsersByIdsRequest};

let client = Client::builder()
    .credentials(Credentials {
        api_key: std::env::var("BRAZE_API_KEY")?,
        rest_endpoint: "https://rest.iad-01.braze.com".into(),
    })
    .build()?;

let response = client
    .export()
    .users_by_ids(&ExportUsersByIdsRequest {
        external_ids: Some(vec!["user_1".into()]),
        fields_to_export: Some(vec!["email".into()]),
        ..Default::default()
    })
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
cargo run --example export_users -- external_user_id_1 external_user_id_2
```

## Coverage

| API | Endpoint | Status |
| --- | --- | --- |
| Export | `POST /users/export/ids` | ✅ |

## Testing

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Integration tests skip silently when `BRAZE_CREDENTIALS` is unset.

## License

MPL-2.0
