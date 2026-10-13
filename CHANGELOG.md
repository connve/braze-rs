# Changelog

All notable changes are documented here. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0]

### Changed
- **Breaking:** no API feature is enabled by default.
  - `braze = "0.1"` → `braze = { version = "0.2", features = ["export"] }`
- **Breaking:** `Credentials::api_key` is a `SecretString`, and `Credentials` no longer implements `Serialize`. `api_key: "...".into()` still works.
- **Breaking:** `export::UserAlias` is now the same type as `messaging::UserAlias`.
- **Breaking:** `export::ExportUsersByIdsRequest`, `ExportUsersByIdsResponse` and `ExportedUser` are the generated types. List fields are `Vec` instead of `Option<Vec>`, and `message` is `Option<String>`.
  - `external_ids: Some(vec![...])` → `external_ids: vec![...]`
- Every operation in the generated client accepts both 200 and 201 as success.
- `Builder::build` trims whitespace around the API key. A key that isn't valid in an HTTP header fails with `Error::InvalidApiKey`; before, every request failed with `MissingApiKey`.
- `Builder::build` rejects a `rest_endpoint` that can't carry a path, such as `mailto:`.
- Request paths are built from percent-encoded segments.
- `tracing` is an optional dependency, enabled by `trace`.
- The `export` usage example moved from the crate root to the `export` module docs.
- Workspace lints live in `[workspace.lints.clippy]`. `clippy::all` is denied, and suppressions must use `#[expect(lint, reason = "...")]`.
- `resolver = "3"`, and `rust-toolchain.toml` pins stable.
- CI: `dtolnay/rust-toolchain` replaces the archived `actions-rs/toolchain`. New `test-msrv` job (Rust 1.88, `--locked`). Clippy runs for each feature with `cargo hack`. The integration job runs every test file. The release job tests with all features.

### Added
- `messaging` feature and `Client::messaging()`:
  - `POST /messages/send`
  - `POST /campaigns/trigger/send`
  - `POST /canvas/trigger/send`
  - `POST /sends/id/create`
  - `POST /transactional/v1/campaigns/{campaign_id}/send`
- `Export::users_by_segment` for `POST /users/export/segment`.
- `Export::users_by_global_control_group` for `POST /users/export/global_control_group`. `fields_to_export` is optional there, since only newer accounts need it.
- `braze::RequestBuilder`, the builder behind every endpoint's type alias.
- All request types implement `Default`, so literals can end with `..Default::default()`.
- Responses keep unknown fields in `extra`, and messaging responses include Braze's non-fatal `errors`. `ExportedUser::custom_attributes` accepts `null`.
- `Error::is_retryable` docs warn that retrying a messaging send can deliver it twice.
- `Recipient` / `CanvasRecipient::send_to_existing_only` is an `Option<bool>` and is left out of the request unless set. Braze rejects it for alias recipients.
- `.header(name, value)` and `.headers(map)` on every request builder, with `reqwest::RequestBuilder` semantics.
- `Builder::default_headers(HeaderMap)`. Per-call headers with the same name take precedence.
- `Error::InvalidHeader { name }` for an invalid header or a reserved one (`Authorization`, `Content-Type`, `Content-Length`, `Transfer-Encoding`, `Accept`, `Host`, `User-Agent`). Returned before any request is sent.
- `Error::InvalidPathSegment { segment }` for an ID that is empty, `.`, `..`, or contains control characters. Returned before any request is sent.
- Re-exports: `braze::header` (`reqwest::header`), `braze::chrono`, `braze::secrecy`.
- docs.rs builds with all features and shows which feature each item needs.
- `trigger_campaign` and `export_segment` examples.
- Mock tests for every endpoint. Live tests only use unknown IDs, so they never send a message or start an export.
- `.githooks/pre-commit`: format check, clippy (each feature via `cargo hack` when installed, and all features), tests, `cargo audit`. It refuses to run with unstaged changes. Enable with `git config core.hooksPath .githooks`.

## [0.1.0] - 2026-06-23

### Added
- Initial release of the unofficial Rust SDK for Braze APIs.
- Workspace split into `braze` (user-facing wrapper) and `braze_generated` (`progenitor`-built client from a hand-maintained `openapi.yaml`).
- `Client` + `Builder` + `Credentials` with API-key bearer auth. Credentials load from a JSON file pointed to by `BRAZE_CREDENTIALS`.
- `Error` enum with `is_retryable()` covering transport, API, deserialize, and credential failures. Retryable on 408 / 425 / 429 / 5xx and on transport timeout / connect / request errors.
- Shared HTTP plumbing in `crate::http` (`post_json`, `build_reqwest_client`, `bearer_header`) reused across endpoint modules. One `reqwest::Client` is built per `braze::Client` and shared via `Arc<Inner>`.
- `export` feature (default): `Client::export().users_by_ids(&req).send().await` wrapping `POST /users/export/ids`, with `ExportUsersByIdsRequest` / `ExportUsersByIdsResponse` / `ExportedUser` / `UserAlias` typed models. Unknown response fields are preserved via `#[serde(flatten)] extra: BTreeMap<String, Value>`.
- `braze::export::UsersByIds` — reqwest-style request builder returned by `users_by_ids(&req)`, dispatched with `.send().await`. Marked `#[must_use]` so forgetting `.send()` is a compile warning. Sets the shape for future per-call options and headers without further breaking changes.
- `trace` feature gates `#[tracing::instrument(skip_all)]` on public async methods. `tracing` is pulled in with the `attributes` feature so the attribute macro resolves.
- Curated OpenAPI spec at `generated/braze/openapi.yaml`, normalised from the [braze-community](https://github.com/braze-community/braze-specification) spec: explicit `operationId`, `BearerAuth` security per operation, typed response schemas, and request schemas promoted to `components/schemas/`.
- `generated/braze/build.rs` runs `progenitor::Generator` at build time; output lives in `OUT_DIR/generated.rs` and is included from `braze_generated::lib`.
- Runnable example at `examples/braze/export_users.rs` (`cargo run --example export_users -- <external_id>...`).
- Integration test scaffold (`braze/tests/common.rs` + `skip_if_no_credentials!` macro + `client()` helper) — wired through `[[test]]` with `required-features = ["export"]`; skips silently when `BRAZE_CREDENTIALS` is unset.
- Integration tests for the export endpoint: unknown `external_id` lands in `invalid_user_ids`; request with no identifiers maps to `Error::Api { status: 400, .. }`.
- Offline mock tests (`braze/tests/mock_export.rs`) using `wiremock` — stub `POST /users/export/ids` so contributors can exercise the wrapper → HTTP → serde path without a live Braze account. Two happy-path checks: request shape / bearer / body match + 200 response deserialisation; 400 response surfacing through `Error::Api` with `is_retryable() == false`.
- CI workflows: `lint.yml` (clippy default + all-features, rustfmt), `test.yml` (stable + nightly build/test + integration job that materialises `$BRAZE_CREDENTIALS` from a GitHub secret), `security.yml` (`cargo audit` on PR + daily cron), `release.yml` (detect-and-tag from workspace version, extract matching CHANGELOG section, `cargo workspaces publish`).
- `.github/CODEOWNERS`, `.gitignore`, `CHANGELOG.md` in Keep-a-Changelog format.
- `AGENTS.md` covering static API-key auth, `BRAZE_CREDENTIALS` JSON shape, OpenAPI normalisation rules, `is_retryable()` mapping for Braze, and the recipe for adding a new endpoint.
