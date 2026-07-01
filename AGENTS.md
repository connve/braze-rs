# AGENTS.md

Guidance for working on `braze-rs`, the unofficial Rust SDK for Braze APIs.

## Repository shape

```
braze-rs/
├── braze/                  # User-facing wrapper: auth, error, per-API modules
│   ├── src/
│   │   ├── lib.rs
│   │   ├── client.rs       # Client, Builder, Credentials
│   │   ├── error.rs        # Error + is_retryable()
│   │   ├── http.rs         # Shared reqwest plumbing (post_json, etc.)
│   │   ├── export.rs       # /users/export/* wrappers
│   │   └── export/         # per-endpoint request/response types
│   └── tests/              # Integration tests, gated on $BRAZE_CREDENTIALS
├── generated/
│   └── braze/              # progenitor-built client + openapi.yaml + build.rs
└── examples/braze/         # Runnable examples
```

**Split rationale:** the generated crate under `generated/braze` is regenerated whenever the OpenAPI spec changes. Keeping it separate means consumers depend on the ergonomic wrapper, not on generator output. The wrapper re-exports the request/response types callers actually need and hides `braze_generated` behind an escape-hatch re-export at `braze::generated`.

## Auth: static API key

Braze uses a long-lived REST API key sent as `Authorization: Bearer <key>`. There is no token refresh, no expiry, no `access_token()` indirection. The wrapper owns one `reqwest::Client` for the lifetime of the `braze::Client` and injects the bearer header per request. There is no HTTP client cache to invalidate.

### Credentials JSON

Shape and dashboard location are documented in the [README](README.md#credentials). For contributors: integration tests and examples read `BRAZE_CREDENTIALS` (path to the JSON), never individual `*_KEY` / `*_ENDPOINT` env vars. Tests must skip silently when the env var is unset so `cargo test` runs offline.

## Spec: no upstream, hand-curated

Braze does not publish a machine-readable spec. We track a curated subset in `generated/braze/openapi.yaml`, seeded from [braze-community/braze-specification][bcs] (`openapi/spec.json`). Anything copied in **must be normalised** before `progenitor` sees it:

- Add an explicit `operationId` per operation (the upstream spec omits them and progenitor generates ugly fallbacks). Use single-capital acronyms (`listUrls`, not `listURLs`) so snake_case conversion produces `list_urls`, not `list_u_r_ls`.
- Attach `security: [{ BearerAuth: [] }]` per operation.
- Replace upstream `type: object` response stubs with real schemas derived from the Braze docs page for the endpoint.
- Promote inline request schemas to `components/schemas/` so generated type names are readable.

Never edit files under `generated/braze/src/` by hand. Fix the spec, then rebuild.

[bcs]: https://github.com/braze-community/braze-specification

## Wrapper shape

The wrapper owns:

- **Auth**: a single `Client` + `Builder` + `Credentials` (see `braze/src/client.rs`).
- **Per-API modules**: thin wrappers (`export.rs`, future `messaging.rs`, `users.rs`, …) that hold `&Client` and dispatch through `crate::http`.
- **Retry policy**: `Error::is_retryable()` — callers drive their own retry loops. The wrapper never retries on its own.
- **Re-exports**: request / response types are re-exported from the per-API module so callers don't depend on `braze_generated` directly.

## Request builder pattern

Every endpoint method returns a **request builder**, dispatched with `.send().await`. It never returns a bare `Future`.

```rust
let response = client
    .export()
    .users_by_ids(&request)   // returns UsersByIds<'a, 'r> — synchronous
    .send()                   // async fn send(self) -> Result<...>
    .await?;
```

Rules:

- The builder struct is `#[must_use = "request builders do nothing until \`.send().await\` is called"]`.
- `#[cfg_attr(feature = "trace", tracing::instrument(skip_all))]` lives on `send`, not on the accessor.
- Optional per-call parameters become builder methods (`.field(...)`), not `Option<T>` on the top-level accessor.
- When Braze eventually documents an opt-in request header, add `.header(name, value)` / `.headers(HeaderMap)` on the builder, with a `HeaderBag`-style accumulator that defers name/value validation to `.send()`. Forbidden headers (`Authorization`, `Content-Type`, `Accept`) surface as `Error::InvalidHeader { name }` and are not retryable.

This is intentionally set up as the default for every endpoint from 0.1.0 so we never break call sites again for opt-in parameters or headers.

## Error handling

- Every public error enum is `#[non_exhaustive]` and exposes `is_retryable(&self) -> bool`.
- Prefer `#[source]` over `#[from]` — the conversion should be explicit at the call site (`.map_err(|source| Error::Variant { source })`).
- Never `unwrap()`, `expect()`, or `panic!()` in production code. `expect()` in tests is fine.
- No `Generic(String)` catch-all — one variant per failure mode.

`is_retryable()` mapping for Braze:

| Variant | Retryable | Reason |
| --- | --- | --- |
| `Http { source }` | `source.is_timeout() / is_connect() / is_request()` | transport flake |
| `Api { status, .. }` | 5xx OR 408 OR 425 OR 429 | Braze rate-limits at 429; 5xx is transient |
| `Api { status, .. }` (other 4xx) | no | client error, retry won't help |
| `MissingApiKey`, `InvalidEndpoint`, `CredentialsIo`, `CredentialsParse`, `Deserialize` | no | configuration / contract failure |

## Adding a new endpoint

1. **Spec**: copy the operation from the braze-community spec, normalise it (see above), append to `generated/braze/openapi.yaml`. `cargo build -p braze_generated` must succeed.
2. **Wrapper module**: add `braze/src/<api>.rs` (with `braze/src/<api>/` submodules for request/response types). The accessor returns a builder struct; `send` calls `crate::http::post_json` or a sibling helper. Gate behind a Cargo feature in `braze/Cargo.toml` if it's an opt-in surface.
3. **Builder**: `#[must_use]`, `async fn send(self) -> Result<Response>`, `#[cfg_attr(feature = "trace", tracing::instrument(skip_all))]` on `send`.
4. **Re-export**: surface request / response types from the wrapper module so callers don't depend on `braze_generated` directly.
5. **Tests**: add `braze/tests/<api>.rs` using `mod common;` + `skip_if_no_credentials!()`. Wire it in `braze/Cargo.toml`:

   ```toml
   [[test]]
   name = "<api>"
   path = "tests/<api>.rs"
   required-features = ["<api>"]
   ```

6. **CHANGELOG**: append the user-visible change under `[Unreleased]` (or the pre-release section if 0.1.0 hasn't shipped yet).

## Code style

- **Module structure (Rust 2018+):** `module.rs` + `module/submodule.rs`. Never `module/mod.rs`.
- **Time/date:** `chrono`. Never compute timestamps or timezones manually.
- **Errors:** `thiserror` in the wrapper, `anyhow` only in examples / binaries.
- **HTTP:** `reqwest` with `rustls`, no default features.
- **Serde:** `#[serde(default)]` for optional response fields; `#[serde(skip_serializing_if = "Option::is_none")]` for optional request fields. Unknown response fields land in a `#[serde(flatten)] extra: BTreeMap<String, Value>` so we never drop data when Braze extends a payload.
- **Instrumentation:** `#[cfg_attr(feature = "trace", tracing::instrument(skip_all))]` on every public async method — currently on `.send()` methods of builders.
- **Comments:** default to none. Add one only when the *why* is non-obvious (hidden constraint, workaround, surprising invariant). Don't explain *what* the code does — names should do that. No commented-out code.
- **Rustdoc:** all public APIs get docs. Examples use `# async fn run() -> Result<(), braze::Error>` wrappers, marked `no_run` if they require credentials.
- **No emojis** in code, comments, or commit messages unless explicitly requested.

## Dependency management

- All dependency versions live in the root `[workspace.dependencies]`.
- Member crates use `dep = { workspace = true, features = [...] }`. Never declare versions in individual crates.

## Testing

Test **behaviour**, not implementation:

- **Do test**: validation logic, state transitions, error conditions (via `matches!`), edge cases, serde round-trips that a schema change would break.
- **Don't test**: derive macros (`Clone`, `Debug`, `PartialEq`), string formatting (`format!("{:?}")`), constants, trivial getters, `Display` error messages.

```rust
// Good
assert!(matches!(result, Err(Error::MissingApiKey)));

// Bad
assert_eq!(error.to_string(), "API key is required to call Braze");
```

Integration tests live in `braze/tests/`, use `mod common;`, and start with `skip_if_no_credentials!();`. They skip silently when `BRAZE_CREDENTIALS` is unset so `cargo test` runs offline.

## Quality bar

Before any change ships:

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`

## Git & releases

- Commits: conventional-commit style (`feat:`, `fix:`, `chore:`). Be specific about *why*, not just *what*.
- Never commit unless explicitly asked.
- Release flow: bump version via `cargo workspaces version`, merge to `main`, the release workflow tags and publishes to crates.io.

## What lives where

- **`AGENTS.md`** (this file): guidance for working on the repo.
- **`README.md`**: user-facing — what the crate does, how to install and use it.
- **`CHANGELOG.md`**: per-release notes, Keep-a-Changelog format.
