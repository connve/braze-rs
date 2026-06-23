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
    /// See the [Braze docs][docs] for the rules around mixing identifier
    /// types and which fields are returned.
    ///
    /// [docs]: https://www.braze.com/docs/api/endpoints/export/user_data/post_users_identifier/
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn users_by_ids(
        &self,
        request: &ExportUsersByIdsRequest,
    ) -> Result<ExportUsersByIdsResponse> {
        crate::http::post_json(
            self.client.http(),
            self.client.base_url(),
            self.client.api_key(),
            "users/export/ids",
            request,
        )
        .await
    }
}
