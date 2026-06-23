# AGENTS.md

Guidance for working on `braze-rs`.

This repo is a sibling SDK in the salesforce-rs family. The recipe lives in
[connve/salesforce-rs/AGENTS.md][sf-agents] — read it first. This file only
records the deltas that are specific to wrapping Braze.

[sf-agents]: https://github.com/connve/salesforce-rs/blob/main/AGENTS.md

## Repository shape

```
braze-rs/
├── braze/                  # User-facing wrapper: auth, error, per-API modules
│   ├── src/
│   │   ├── lib.rs
│   │   ├── client.rs       # Client, Builder, Credentials
│   │   ├── error.rs        # Error + is_retryable()
│   │   ├── http.rs         # Shared reqwest plumbing (post_json, etc.)
│   │   └── export.rs       # POST /users/export/* wrappers
│   └── tests/              # Integration tests gated on $BRAZE_CREDENTIALS
├── generated/
│   └── braze/              # progenitor-built client + openapi.yaml + build.rs
└── examples/braze/         # Runnable examples
```

## Deltas from the salesforce-rs recipe

### Auth: static API key, not OAuth2
Braze uses a long-lived REST API key sent as `Authorization: Bearer <key>`.
There is no token refresh, no expiry, no `access_token()` indirection. The
wrapper owns one `reqwest::Client` for the lifetime of the `braze::Client`
and injects the bearer header per request.

This is why `braze/src/http.rs` is simpler than salesforce-rs's
`HttpClientCache`: there is nothing to invalidate.

### Credentials JSON
Loaded from the path in `BRAZE_CREDENTIALS`. Shape:

```json
{
  "api_key": "YOUR-REST-API-KEY",
  "rest_endpoint": "https://rest.iad-01.braze.com"
}
```

`rest_endpoint` is the per-instance REST URL from the Braze dashboard
(*Settings → API Keys*). It is part of the credential, not a separate
config field — different keys can target different instances.

### No upstream OpenAPI spec
Braze does not publish a machine-readable spec. We track a curated subset
in `generated/braze/openapi.yaml`, seeded from
[braze-community/braze-specification][bcs] (`openapi/spec.json`). Anything
copied in **must be normalised** before `progenitor` sees it:

- Add an explicit `operationId` per operation (the upstream spec omits them
  and progenitor generates ugly fallbacks).
- Attach `security: [{ BearerAuth: [] }]` per operation.
- Replace the upstream `type: object` response stubs with real schemas
  derived from the Braze docs page for the endpoint.
- Promote inline request schemas to `components/schemas/` so generated
  type names are readable.

[bcs]: https://github.com/braze-community/braze-specification

### `is_retryable()` mapping for Braze
Per AGENTS.md §6, every public `Error` variant is classified:

| Variant | Retryable | Reason |
| --- | --- | --- |
| `Http { source }` | `source.is_timeout() / is_connect() / is_request()` | transport flake |
| `Api { status, .. }` | 5xx OR 408 OR 425 OR 429 | Braze rate-limits at 429; 5xx is transient |
| `Api { status, .. }` (other 4xx) | no | client error, retry won't help |
| `MissingApiKey`, `InvalidEndpoint`, `CredentialsIo`, `CredentialsParse`, `Deserialize` | no | configuration / contract failure |

The wrapper never retries on its own — callers decide.

## Adding a new endpoint

1. **Spec**: copy the operation from the braze-community spec, normalise it
   (see above), append to `generated/braze/openapi.yaml`. `cargo build -p
   braze_generated` should succeed.
2. **Wrapper module**: add `braze/src/<api>.rs` (and `braze/src/<api>/`
   submodules) that hold a `&Client` and call `crate::http::post_json` (or
   sibling helper). Gate behind a Cargo feature in `braze/Cargo.toml` if
   it's an opt-in surface.
3. **Re-export**: surface request/response types from the wrapper module so
   callers don't depend on `braze_generated` directly.
4. **Tests**: add `braze/tests/<api>.rs` using `mod common;` +
   `skip_if_no_credentials!()`. Wire it in the wrapper's `Cargo.toml`:

   ```toml
   [[test]]
   name = "<api>"
   path = "tests/<api>.rs"
   required-features = ["<api>"]
   ```

5. **CHANGELOG**: append the user-visible change under `[Unreleased]`.

## Quality bar (mirrors salesforce-rs)

Before any change ships:

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`

Integration tests skip silently when `BRAZE_CREDENTIALS` is unset, so the
above runs offline.
