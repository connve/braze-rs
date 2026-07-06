//! Export endpoints (`/users/export/*`).

mod users;

pub use users::{ExportUsersByIdsRequest, ExportUsersByIdsResponse, ExportedUser, UserAlias};

use crate::client::Client;
use crate::error::Result;

/// Accessor returned by [`Client::export`](crate::Client::export).
#[derive(Debug)]
pub struct Export<'a> {
    client: &'a Client,
}

impl<'a> Export<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `POST /users/export/ids` — export user profiles by identifier.
    ///
    /// Returns a request builder; dispatch with [`UsersByIds::send`].
    ///
    /// See the [Braze docs][docs] for the rules around mixing identifier
    /// types and which fields are returned.
    ///
    /// [docs]: https://www.braze.com/docs/api/endpoints/export/user_data/post_users_identifier/
    pub fn users_by_ids<'r>(&self, request: &'r ExportUsersByIdsRequest) -> UsersByIds<'a, 'r> {
        UsersByIds {
            client: self.client,
            request,
        }
    }
}

/// Builder for [`Export::users_by_ids`].
#[must_use = "request builders do nothing until `.send().await` is called"]
#[derive(Debug)]
pub struct UsersByIds<'a, 'r> {
    client: &'a Client,
    request: &'r ExportUsersByIdsRequest,
}

impl<'a, 'r> UsersByIds<'a, 'r> {
    /// Dispatches the request.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn send(self) -> Result<ExportUsersByIdsResponse> {
        crate::http::post_json(
            self.client.http(),
            self.client.base_url(),
            self.client.api_key(),
            "users/export/ids",
            self.request,
        )
        .await
    }
}
