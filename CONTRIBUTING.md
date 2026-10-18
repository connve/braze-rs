# Contributing to braze-rs

## Repository shape

```
braze-rs/
├── braze/                  # User-facing crate
│   ├── src/
│   │   ├── lib.rs          # Re-exports, crate docs
│   │   ├── client.rs       # Client, Builder, Credentials
│   │   ├── error.rs        # Error
│   │   ├── http.rs         # reqwest plumbing, paths, rate-limit headers
│   │   ├── headers.rs      # HeaderBag, reserved headers
│   │   ├── request.rs      # RequestBuilder
│   │   └── <api>.rs        # One module per feature
│   └── tests/              # Live tests and mock_* tests
├── generated/
│   └── braze/              # openapi.yaml, build.rs
└── examples/braze/
```

Callers use `braze`; `braze_generated` is re-exported as `braze::generated`
for endpoints the wrapper doesn't cover yet.

## Auth

`Authorization: Bearer <key>`. A key belongs to one workspace, and each Braze
instance has its own REST endpoint, so `base_url` is required.

The SDK takes both as values (`Builder::api_key` / `Builder::base_url`, or
`Credentials`, which deserializes from `{ "api_key": "..." }`). It never reads
files or environment variables; where secrets come from is the application's
call. Tests and examples read `BRAZE_API_KEY` and `BRAZE_BASE_URL`.

## Spec

Braze doesn't publish a spec. `generated/braze/openapi.yaml` is a curated
subset of [braze-community/braze-specification][bcs], normalised before
progenitor sees it:

- Every operation has an `operationId`, with single-capital acronyms
  (`listUrls`, not `listURLs`).
- Every operation has `security: [{ BearerAuth: [] }]`.
- Response schemas are written from the Braze docs page of the endpoint;
  upstream has only `type: object` stubs.
- Request schemas live in `components/schemas/`, so generated names are
  readable.
- Response objects use `additionalProperties: {}`, so typify adds an `extra`
  field. With `true` it drops unknown fields.
- Optional request fields have no `default:`; typify would always serialize
  them.
- Operations accept both 200 and 201.

`build.rs` patches `Default` onto every request body type with a required
field. Don't edit the generated output; change the spec or `build.rs`.

[bcs]: https://github.com/braze-community/braze-specification

## Request builders

Endpoint methods return `RequestBuilder<'a, 'r, Req, Resp>` under a
per-endpoint alias. It has `.header()`, `.headers()` and `.send()`, and is
`#[must_use]`.

- Pass caller-supplied IDs as path segments, never `format!` them into a
  path. Segments are validated and percent-encoded.
- Optional per-call parameters are builder methods, not `Option` arguments.
- Headers follow `reqwest::RequestBuilder`: `.header()` appends,
  `.headers(map)` replaces only the names in `map`, and errors surface from
  `.send()`. Per-call headers win over `Builder::default_headers`.
- Reserved headers: `Authorization`, `Content-Type`, `Content-Length`,
  `Transfer-Encoding`, `Accept`, `Host`, `User-Agent`.

## Errors

| Variant | Retryable |
| --- | --- |
| `Http` | on timeout, connect, request errors |
| `Api` | 5xx, 408, 425, 429 |
| everything else | no |

On a 429, `Error::retry_after()` reads `X-RateLimit-Retry-After`, or
`X-RateLimit-Reset` if it's missing. `Error::Api`'s
message leaves out `body`, which can contain personal data. Messaging sends
have no idempotency key, so a retry after a timeout or 5xx can send twice.

- Every public error enum is `#[non_exhaustive]` and has `is_retryable()`.
- One variant per failure mode; no `Generic(String)`.
- `#[source]` with `.map_err(|source| Error::Variant { source })`, not
  `#[from]`.
- No `unwrap()`, `expect()` or `panic!()` outside tests.

## Adding an endpoint

1. Copy the operation from the braze-community spec into `openapi.yaml` and
   normalise it. `cargo build -p braze_generated` must pass.
2. Add the method and its builder alias to `braze/src/<api>.rs`, calling
   `RequestBuilder::new(client, vec![...segments], request)`. Re-export the
   request and response types from the module. A new module gets a feature
   enabling `_api`.
3. Tests:
   - `tests/<api>.rs`: live tests against a test workspace. Checks use
     `ensure!` / `ensure_rejected!` from `common.rs`. Endpoints with side
     effects only get rejection paths (unknown IDs from `unique_name!`,
     `send_to_existing_only`), so CI never delivers a message or starts an
     export.
   - `tests/mock_<api>.rs`: `wiremock` tests of path, headers, and body.

   Live tests are `#[ignore = "needs a Braze test workspace in BRAZE_API_KEY
   and BRAZE_BASE_URL"]`; mock tests aren't. Each test file opens with a
   `//!` saying what it covers, what it needs, and why it is or isn't
   `#[ignore]`d.

   Build requests as typed literals ending in `..Default::default()`;
   `json!` is only for expected bodies and stubbed responses in mock tests.

   Register both in `braze/Cargo.toml` with `required-features`.
4. Put the usage example in the module docs, and add a runnable example
   under `examples/braze/` if it helps.
5. Bump the version and add a CHANGELOG entry in the same PR.

## Code style

- `module.rs` + `module/submodule.rs`, never `mod.rs`.
- `thiserror` in the library, `anyhow` only in examples.
- Comments only for a non-obvious *why*. Public items have rustdoc;
  examples needing credentials are `no_run`.
- Dates and times use `chrono`, never hand-computed from `SystemTime`.
- `#[cfg_attr(feature = "trace", tracing::instrument(skip_all))]` on public
  async methods.
- Re-export dependency types that appear in the public API (`braze::chrono`,
  `braze::header`, `braze::secrecy`).
- Lint suppressions are `#[expect(lint, reason = "...")]`; `#[allow]` is
  denied. `generated/braze` doesn't use the workspace lints, since its code
  is emitted by progenitor.
- Test behaviour, not derives, formatting or constants. Match errors with
  `matches!`, not message strings.

## Dependencies

- All versions live in the root `[workspace.dependencies]`.
- `braze_generated` is pinned with `=x.y.z`; a release bumps both.
- `tokio` is a dev-dependency only.
- MSRV is 1.88, the minimum required by `progenitor` 0.13. Raise it only when
  a dependency requires it, together with the `test-msrv` job and a CHANGELOG
  entry.
- Every single-feature build must pass clippy without `#[expect]`; gate
  shared code on `_api`.

## Checks

```bash
cargo fmt --all
cargo hack clippy -p braze --each-feature --skip _api --all-targets -- -D warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo package --workspace --locked
```

Live tests, against a test workspace:

```bash
BRAZE_API_KEY=... BRAZE_BASE_URL=https://rest.iad-01.braze.com \
  cargo test -p braze --all-features --tests -- --ignored
```

The pre-commit hook runs most of these. It refuses to run with unstaged
changes, since it checks the working tree:

```bash
git config core.hooksPath .githooks
```

## Releases

Conventional commits (`feat:`, `fix:`, `chore:`). Bump with
`cargo workspaces version --no-git-commit --yes <minor|patch>` and merge to
`main`; the release workflow tags and publishes.
