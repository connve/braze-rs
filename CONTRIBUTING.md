# Contributing to braze-rs

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
│   │   ├── headers.rs      # HeaderBag, reserved headers
│   │   ├── request.rs      # RequestBuilder used by every endpoint
│   │   ├── export.rs       # /users/export/* wrappers
│   │   └── messaging.rs    # /messages, /campaigns|canvas/trigger, /sends, /transactional
│   └── tests/              # Live tests (skipped without $BRAZE_CREDENTIALS) and mock tests
├── generated/
│   └── braze/              # progenitor-built client + openapi.yaml + build.rs
└── examples/braze/         # Runnable examples
```

**Split rationale:** the generated crate under `generated/braze` is regenerated whenever the OpenAPI spec changes. Keeping it separate means consumers depend on the ergonomic wrapper, not on generator output. The wrapper re-exports the request/response types callers actually need and hides `braze_generated` behind an escape-hatch re-export at `braze::generated`.

## Auth: static API key

Braze uses a long-lived REST API key sent as `Authorization: Bearer <key>`. There is no token refresh, no expiry, no `access_token()` indirection. The wrapper owns one `reqwest::Client` for the lifetime of the `braze::Client` and injects the bearer header per request. There is no HTTP client cache to invalidate.

Integration tests and examples read `BRAZE_CREDENTIALS` (path to the JSON), never individual `*_KEY` / `*_ENDPOINT` env vars. Tests must skip silently when the env var is unset so `cargo test` runs offline.

## Spec: no upstream, hand-curated

Braze does not publish a machine-readable spec. We track a curated subset in `generated/braze/openapi.yaml`, seeded from [braze-community/braze-specification][bcs] (`openapi/spec.json`). Anything copied in **must be normalised** before `progenitor` sees it:

- Add an explicit `operationId` per operation (the upstream spec omits them and progenitor generates ugly fallbacks). Use single-capital acronyms (`listUrls`, not `listURLs`) so snake_case conversion produces `list_urls`, not `list_u_r_ls`.
- Attach `security: [{ BearerAuth: [] }]` per operation.
- Replace upstream `type: object` response stubs with real schemas derived from the Braze docs page for the endpoint.
- Promote inline request schemas to `components/schemas/` so generated type names are readable.
- Use `additionalProperties: {}` on response objects. typify then adds an `extra` field; with `true` it drops unknown fields.
- Don't set `default:` on optional request fields. typify then always serializes them.

Never edit files under `generated/braze/src/` by hand. Fix the spec, then rebuild.

[bcs]: https://github.com/braze-community/braze-specification

## Wrapper shape

The wrapper owns:

- **Auth**: a single `Client` + `Builder` + `Credentials` (see `braze/src/client.rs`).
- **Per-API modules**: thin wrappers (`export.rs`, `messaging.rs`, future `users.rs`, …) that hold `&Client` and dispatch through `crate::http`.
- **Retry policy**: `Error::is_retryable()` — callers drive their own retry loops. The wrapper never retries on its own. Messaging sends aren't idempotent, so a retry after a timeout or 5xx can send twice.
- **Re-exports**: request / response types are re-exported from the per-API module so callers don't depend on `braze_generated` directly.

## Request builder pattern

Every endpoint method returns a **request builder**, dispatched with `.send().await`. It never returns a bare `Future`.

```rust
let response = client
    .export()
    .users_by_ids(&request)   // UsersByIds<'a, 'r>, an alias of RequestBuilder
    .send()                   // async fn send(self) -> Result<...>
    .await?;
```

Rules:

- Endpoint methods return `RequestBuilder<'a, 'r, Req, Resp>` (`braze/src/request.rs`) under a per-endpoint type alias. It provides `.header()`, `.headers()` and `.send()`, and is `#[must_use]`.
- Optional per-call parameters become builder methods (`.field(...)`), not `Option<T>` on the top-level accessor.
- Request types implement `Default`. `generated/braze/build.rs` adds it to every request body type with a required field. Docs and tests build literals with `..Default::default()`.

Header semantics follow `reqwest::RequestBuilder` so they hold no surprises for reqwest users:

- `.header()` appends; `.headers(map)` merges, replacing only the names present in `map`.
- Name/value conversion errors are deferred to `.send()`, so chaining never returns a `Result`.
- Per-call headers take precedence over `Builder::default_headers` with the same name.
- Reserved headers (`Authorization`, `Content-Type`, `Content-Length`, `Transfer-Encoding`, `Accept`, `Host`, `User-Agent`) surface as `Error::InvalidHeader { name }` before any request is sent, and are not retryable.
- `HeaderMap` / `HeaderValue` appear in the public API, so they are re-exported as `braze::header`.

**Reference implementations:** `braze/src/request.rs`, `braze/src/headers.rs` (`HeaderBag`), `braze/tests/mock_messaging.rs` (`per_call_headers_merge_over_client_defaults`).

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
| `MissingApiKey`, `InvalidApiKey`, `InvalidEndpoint`, `CredentialsIo`, `CredentialsParse`, `Deserialize` | no | configuration / contract failure |
| `InvalidHeader { name }`, `InvalidPathSegment { segment }` | no | rejected before the request is sent |

## Adding a new endpoint

1. **Spec**: copy the operation from the braze-community spec, normalise it (see above), append to `generated/braze/openapi.yaml`. `cargo build -p braze_generated` must succeed.
2. **Wrapper module**: add `braze/src/<api>.rs`. Never `format!` caller-supplied IDs into a path string; pass them as path segments, which are validated and percent-encoded for you. Gate the module behind a Cargo feature named after it in `braze/Cargo.toml` (no API feature is on by default), and make it enable `_api` (`<api> = ["_api"]`). Shared plumbing is gated on `_api`.
3. **Builder**: add a type alias of `RequestBuilder` and an accessor that calls `RequestBuilder::new(client, vec![...segments], request)`, e.g. `vec!["campaigns", campaign_id, "send"]`.
4. **Re-export**: surface request / response types from the wrapper module so callers don't depend on `braze_generated` directly.
5. **Tests**: two files per endpoint.
   - `braze/tests/<api>.rs` — live integration tests using `mod common;` + `skip_if_no_credentials!()`. Skipped silently when `BRAZE_CREDENTIALS` is unset. For endpoints with side effects (sends, writes) only exercise rejection paths (unknown IDs, `send_to_existing_only`) so CI never delivers a message. The `integration-test` CI job runs `--test '*'`, so a new file is picked up without editing the workflow.
   - `braze/tests/mock_<api>.rs` — offline mock tests using `wiremock` (dev-dep). At minimum: one happy-path request+response check. These run on every `cargo test` so external contributors get coverage without a Braze account.

   Wire both in `braze/Cargo.toml`:

   ```toml
   [[test]]
   name = "<api>"
   path = "tests/<api>.rs"
   required-features = ["<api>"]

   [[test]]
   name = "mock_<api>"
   path = "tests/mock_<api>.rs"
   required-features = ["<api>"]
   ```

6. **Example**: add `examples/braze/<name>.rs` wired via `[[example]]` with `required-features = ["<api>"]`.
7. **Docs**: put the per-API usage example in the module-level docs (`//!` in `braze/src/<api>.rs`), not in `lib.rs`, so it drops out of the build cleanly when the feature is off.
8. **Version + CHANGELOG**: see [Versioning and the changelog](#versioning-and-the-changelog).

**Reference implementations:**
- Builder + segment-based path with a caller-supplied ID: `braze/src/messaging.rs` (`SendTransactionalMessage`)
- Generated types with an `extra` catch-all: `ExportUsersByIdsResponse`, `ExportedUser`
- Side-effect-free live tests: `braze/tests/messaging.rs`
- Mock tests including path-encoding and 4xx surfacing: `braze/tests/mock_messaging.rs`

## Code style

- **Module structure (Rust 2018+):** `module.rs` + `module/submodule.rs`. Never `module/mod.rs`.
- **Time/date:** `chrono`. Never compute timestamps or timezones manually.
- **Errors:** `thiserror` in the wrapper, `anyhow` only in examples / binaries.
- **HTTP:** `reqwest` with `rustls`, no default features.
- **Serde:** `#[serde(default)]` for optional response fields; `#[serde(skip_serializing_if = "Option::is_none")]` for optional request fields. Unknown response fields land in a `#[serde(flatten)] extra: BTreeMap<String, Value>` so we never drop data when Braze extends a payload.
- **Instrumentation:** `#[cfg_attr(feature = "trace", tracing::instrument(skip_all))]` on every public async method — currently on `.send()` methods of builders.
- **Comments:** default to none. Add one only when the *why* is non-obvious (hidden constraint, workaround, surprising invariant). Don't explain *what* the code does — names should do that. No commented-out code.
- **Rustdoc:** all public APIs get docs. Async examples use `# #[tokio::main]` + `# async fn main() -> Result<(), Box<dyn std::error::Error>>`, marked `no_run` if they require credentials.
- **Public dependency types:** if a dependency's type appears in the public API, re-export it (as `braze::chrono`, `braze::header` and `braze::secrecy` are) so callers don't add a second, mismatched version.
- **Feature-gated internals:** gate shared plumbing with `#[cfg(feature = "_api")]`, not with suppressions. A no-feature build must pass clippy without any `#[expect]`.
- **No emojis** in code, comments, or commit messages unless explicitly requested.

## Dependency management

- All dependency versions live in the root `[workspace.dependencies]`.
- Member crates use `dep = { workspace = true, features = [...] }`. Never declare versions in individual crates.
- `braze_generated` is pinned by exact version in `[workspace.dependencies]` alongside its `path`. A release bump has to update both, or the workspace stops resolving.
- We use `resolver = "3"`, so `cargo update` prefers versions that build on our `rust-version`. The `test-msrv` CI job checks this with `--locked`.
- The toolchain is pinned to stable via `rust-toolchain.toml`.
- MSRV is 1.88, the minimum required by `progenitor` 0.13. Raise it only when a dependency requires it, together with the `test-msrv` job and a CHANGELOG entry.

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

## Linting

Lint levels live in `[workspace.lints.clippy]` in the root `Cargo.toml`, and
`braze` opts in with `[lints] workspace = true`. `clippy::all` is denied.

Write suppressions as `#[expect(lint, reason = "...")]` on the smallest item
that needs them — `allow_attributes` and `allow_attributes_without_reason` are
denied, so clippy will point you there. `#[expect]` is the required form
because clippy reports it as unfulfilled once the underlying lint stops
firing, so a suppression that has outlived its cause becomes an error instead
of lingering.

Fix `result_large_err` by boxing the oversized source (`source: Box<...>`);
the variant still exposes the same `#[source]` accessor.

`generated/braze` does not opt into workspace lints: its suppressions are
emitted by progenitor and would come back on the next regeneration.

## Quality bar

Before any change ships:

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings` (slim build, no API features)
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`

The pre-commit hook in `.githooks/` runs fmt, clippy, tests, and `cargo audit`
(if installed). Enable it once per clone:

```bash
git config core.hooksPath .githooks
```

The hook refuses to run with unstaged changes or untracked source files, since it checks the working tree.

## Git & releases

- Commits: conventional-commit style (`feat:`, `fix:`, `chore:`). Be specific about *why*, not just *what*.
- Commit when you're explicitly asked to.
- Release flow: bump version, merge to `main`, the release workflow tags and publishes to crates.io.

### Versioning and the changelog

Every PR that changes the public API carries its own version bump and
changelog entry — both belong in the PR, not in a later release-time pass.

- New public API (a method, a builder, a re-exported type) is a **minor** bump.
  Bug fixes with no signature change are a **patch**. A changed or removed
  signature is a breaking change: bump minor while pre-1.0 and mark the entry
  **Breaking:** with a before/after migration line.
- Bump with `cargo workspaces version --no-git-commit --yes <minor|patch>`,
  then confirm that `[workspace.package] version` and the pinned
  `braze_generated` version in `[workspace.dependencies]` agree.
- Add the entry under a new `## [x.y.z]` heading in
  `CHANGELOG.md`, grouped **Added** / **Changed** / **Fixed**. Describe what a
  consumer can now do.

## What lives where

- **`CONTRIBUTING.md`** (this file): contributor-facing conventions.
- **`README.md`**: user-facing — what the crate does, how to install and use it.
- **`CHANGELOG.md`**: per-release notes.
