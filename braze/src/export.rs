//! Export endpoints (`/users/export/*`).
//!
//! ```no_run
//! use braze::{export::ExportUsersByIdsRequest, Client};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::builder()
//!     .credentials_file(std::env::var("BRAZE_CREDENTIALS")?)?
//!     .build()?;
//!
//! let response = client
//!     .export()
//!     .users_by_ids(&ExportUsersByIdsRequest {
//!         external_ids: vec!["user_1".into()],
//!         fields_to_export: vec!["email".into(), "first_name".into()],
//!         ..Default::default()
//!     })
//!     .send()
//!     .await?;
//!
//! println!("{} users returned", response.users.len());
//! # Ok(())
//! # }
//! ```

pub use braze_generated::types::{
    ExportOutputFormat, ExportUsersByGlobalControlGroupRequest,
    ExportUsersByGlobalControlGroupResponse, ExportUsersByIdsRequest, ExportUsersByIdsResponse,
    ExportUsersBySegmentRequest, ExportUsersBySegmentResponse, ExportedUser, UserAlias,
};

use crate::client::Client;
use crate::request::RequestBuilder;

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
    /// Returns a request builder; dispatch with [`RequestBuilder::send`].
    ///
    /// Set at least one identifier. Braze allows up to 50 `external_ids` or
    /// `user_aliases` per call, but only one `device_id`, `email_address` or
    /// `phone`. Accounts created after 2024-08-22 must set
    /// `fields_to_export`. See the [Braze docs][docs].
    ///
    /// [docs]: https://www.braze.com/docs/api/endpoints/export/user_data/post_users_identifier/
    pub fn users_by_ids<'r>(&self, request: &'r ExportUsersByIdsRequest) -> UsersByIds<'a, 'r> {
        RequestBuilder::new(self.client, vec!["users", "export", "ids"], request)
    }

    /// `POST /users/export/segment` — start an export of every user in a
    /// segment.
    ///
    /// Returns a request builder; dispatch with [`RequestBuilder::send`].
    ///
    /// The export runs asynchronously. Files go to your configured S3, Azure
    /// or GCS bucket; without one, the response carries a temporary `url`.
    /// Braze returns 429 if an export of the same segment is already running.
    /// See the [Braze docs][docs].
    ///
    /// [docs]: https://www.braze.com/docs/api/endpoints/export/user_data/post_users_segment/
    pub fn users_by_segment<'r>(
        &self,
        request: &'r ExportUsersBySegmentRequest,
    ) -> UsersBySegment<'a, 'r> {
        RequestBuilder::new(self.client, vec!["users", "export", "segment"], request)
    }

    /// `POST /users/export/global_control_group` — start an export of every
    /// user in the Global Control Group.
    ///
    /// Returns a request builder; dispatch with [`RequestBuilder::send`].
    ///
    /// The export runs asynchronously, with the same delivery as
    /// [`Export::users_by_segment`]. See the [Braze docs][docs].
    ///
    /// [docs]: https://www.braze.com/docs/api/endpoints/export/user_data/post_users_global_control_group/
    pub fn users_by_global_control_group<'r>(
        &self,
        request: &'r ExportUsersByGlobalControlGroupRequest,
    ) -> UsersByGlobalControlGroup<'a, 'r> {
        RequestBuilder::new(
            self.client,
            vec!["users", "export", "global_control_group"],
            request,
        )
    }
}

/// Builder for [`Export::users_by_ids`].
pub type UsersByIds<'a, 'r> =
    RequestBuilder<'a, 'r, ExportUsersByIdsRequest, ExportUsersByIdsResponse>;

/// Builder for [`Export::users_by_segment`].
pub type UsersBySegment<'a, 'r> =
    RequestBuilder<'a, 'r, ExportUsersBySegmentRequest, ExportUsersBySegmentResponse>;

/// Builder for [`Export::users_by_global_control_group`].
pub type UsersByGlobalControlGroup<'a, 'r> = RequestBuilder<
    'a,
    'r,
    ExportUsersByGlobalControlGroupRequest,
    ExportUsersByGlobalControlGroupResponse,
>;
