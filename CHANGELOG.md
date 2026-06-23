# Changelog

All notable changes are documented here. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-06-23

### Added
- Initial release of the unofficial Rust SDK for Braze APIs.
- Workspace split into `braze` (user-facing wrapper) and `braze_generated` (`progenitor`-built client from a hand-maintained `openapi.yaml`).
- `Client` + `Builder` + `Credentials` with API-key bearer auth. Credentials load from a JSON file pointed to by `BRAZE_CREDENTIALS`.
- `Error` enum with `is_retryable()` covering transport, API, deserialize, and credential failures. Retryable on 408 / 425 / 429 / 5xx and on transport timeout / connect / request errors.
- Shared HTTP plumbing in `crate::http` (`post_json`, `build_reqwest_client`, `bearer_header`) reused across endpoint modules. One `reqwest::Client` is built per `braze::Client` and shared via `Arc<Inner>`.
- `export` feature (default): `Client::export().users_by_ids()` wrapping `POST /users/export/ids`, with `ExportUsersByIdsRequest` / `ExportUsersByIdsResponse` / `ExportedUser` / `UserAlias` typed models. Unknown response fields are preserved via `#[serde(flatten)] extra: BTreeMap<String, Value>`.
- `trace` feature gates `#[tracing::instrument(skip_all)]` on public async methods. `tracing` is pulled in with the `attributes` feature so the attribute macro resolves.
- Curated OpenAPI spec at `generated/braze/openapi.yaml`, normalised from the [braze-community](https://github.com/braze-community/braze-specification) spec: explicit `operationId`, `BearerAuth` security per operation, typed response schemas, and request schemas promoted to `components/schemas/`.
- `generated/braze/build.rs` runs `progenitor::Generator` at build time; output lives in `OUT_DIR/generated.rs` and is included from `braze_generated::lib`.
- Runnable example at `examples/braze/export_users.rs` (`cargo run --example export_users -- <external_id>...`).
- Integration test scaffold (`braze/tests/common.rs` + `skip_if_no_credentials!` macro + `client()` helper) — wired through `[[test]]` with `required-features = ["export"]`; skips silently when `BRAZE_CREDENTIALS` is unset.
- Integration tests for the export endpoint: unknown `external_id` lands in `invalid_user_ids`; request with no identifiers maps to `Error::Api { status: 400, .. }`.
- CI workflows: `lint.yml` (clippy default + all-features, rustfmt), `test.yml` (stable + nightly build/test + integration job that materialises `$BRAZE_CREDENTIALS` from a GitHub secret), `security.yml` (`cargo audit` on PR + daily cron), `release.yml` (detect-and-tag from workspace version, extract matching CHANGELOG section, `cargo workspaces publish`).
- `.github/CODEOWNERS`, `.gitignore` (matches salesforce-rs), `CHANGELOG.md` in Keep-a-Changelog format.
- `AGENTS.md` documenting the deltas from [`connve/salesforce-rs/AGENTS.md`](https://github.com/connve/salesforce-rs/blob/main/AGENTS.md): static API-key auth (no OAuth2), `BRAZE_CREDENTIALS` JSON shape, OpenAPI normalisation rules, `is_retryable()` mapping for Braze, and the 5-step recipe for adding a new endpoint.
