//! Messaging endpoints (`/messages/send`, `/campaigns/trigger/send`, etc.).
//!
//! Request types implement `Default`; end literals with
//! `..Default::default()` so new optional fields don't break your code.
//! Required fields such as `campaign_id` default to empty, and Braze rejects
//! the request if they're left unset.
//!
//! These endpoints aren't idempotent: a retry after a timeout or 5xx can
//! deliver a message twice.
//!
//! ```no_run
//! use braze::{
//!     messaging::{Recipient, SendTriggeredCampaignsRequest},
//!     Client,
//! };
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::builder()
//!     .api_key(std::env::var("BRAZE_API_KEY")?)
//!     .base_url("https://rest.iad-01.braze.com")
//!     .build()?;
//!
//! let response = client
//!     .messaging()
//!     .send_triggered_campaigns(&SendTriggeredCampaignsRequest {
//!         campaign_id: "campaign_id".into(),
//!         recipients: vec![Recipient {
//!             external_user_id: Some("user_1".into()),
//!             ..Default::default()
//!         }],
//!         ..Default::default()
//!     })
//!     .send()
//!     .await?;
//!
//! println!("dispatch_id: {:?}", response.dispatch_id);
//! # Ok(())
//! # }
//! ```

pub use braze_generated::types::{
    Attachment, CanvasRecipient, ConnectedAudience, CreateSendIdsRequest, CreateSendIdsResponse,
    Messages, Recipient, SendMessagesRequest, SendMessagesRequestRecipientSubscriptionState,
    SendMessagesResponse, SendTransactionalMessageRequest, SendTransactionalMessageResponse,
    SendTriggeredCampaignsRequest, SendTriggeredCampaignsResponse, SendTriggeredCanvasesRequest,
    SendTriggeredCanvasesResponse, TransactionalRecipient, UserAlias,
};

use crate::client::Client;
use crate::request::RequestBuilder;

/// Accessor returned by [`Client::messaging`](crate::Client::messaging).
#[derive(Debug)]
pub struct Messaging<'a> {
    client: &'a Client,
}

impl<'a> Messaging<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `POST /messages/send` — send messages immediately using the API only.
    ///
    /// See the [Braze docs][docs].
    ///
    /// [docs]: https://www.braze.com/docs/api/endpoints/messaging/send_messages/post_send_messages/
    pub fn send_messages<'r>(&self, request: &'r SendMessagesRequest) -> SendMessages<'a, 'r> {
        RequestBuilder::new(self.client, vec!["messages", "send"], request)
    }

    /// `POST /campaigns/trigger/send` — send campaigns using API-triggered delivery.
    ///
    /// See the [Braze docs][docs].
    ///
    /// [docs]: https://www.braze.com/docs/api/endpoints/messaging/send_messages/post_send_triggered_campaigns/
    pub fn send_triggered_campaigns<'r>(
        &self,
        request: &'r SendTriggeredCampaignsRequest,
    ) -> SendTriggeredCampaigns<'a, 'r> {
        RequestBuilder::new(self.client, vec!["campaigns", "trigger", "send"], request)
    }

    /// `POST /canvas/trigger/send` — send Canvas messages using API-triggered delivery.
    ///
    /// See the [Braze docs][docs].
    ///
    /// [docs]: https://www.braze.com/docs/api/endpoints/messaging/send_messages/post_send_triggered_canvases/
    pub fn send_triggered_canvases<'r>(
        &self,
        request: &'r SendTriggeredCanvasesRequest,
    ) -> SendTriggeredCanvases<'a, 'r> {
        RequestBuilder::new(self.client, vec!["canvas", "trigger", "send"], request)
    }

    /// `POST /sends/id/create` — create send IDs for tracking message performance.
    ///
    /// See the [Braze docs][docs].
    ///
    /// [docs]: https://www.braze.com/docs/api/endpoints/messaging/send_messages/post_create_send_ids/
    pub fn create_send_ids<'r>(&self, request: &'r CreateSendIdsRequest) -> CreateSendIds<'a, 'r> {
        RequestBuilder::new(self.client, vec!["sends", "id", "create"], request)
    }

    /// `POST /transactional/v1/campaigns/{campaign_id}/send` — send transactional emails.
    ///
    /// See the [Braze docs][docs].
    ///
    /// [docs]: https://www.braze.com/docs/api/endpoints/messaging/send_messages/post_send_transactional_message/
    pub fn send_transactional_message<'r>(
        &self,
        campaign_id: &'r str,
        request: &'r SendTransactionalMessageRequest,
    ) -> SendTransactionalMessage<'a, 'r> {
        RequestBuilder::new(
            self.client,
            vec!["transactional", "v1", "campaigns", campaign_id, "send"],
            request,
        )
    }
}

/// Builder for [`Messaging::send_messages`].
pub type SendMessages<'a, 'r> = RequestBuilder<'a, 'r, SendMessagesRequest, SendMessagesResponse>;

/// Builder for [`Messaging::send_triggered_campaigns`].
pub type SendTriggeredCampaigns<'a, 'r> =
    RequestBuilder<'a, 'r, SendTriggeredCampaignsRequest, SendTriggeredCampaignsResponse>;

/// Builder for [`Messaging::send_triggered_canvases`].
pub type SendTriggeredCanvases<'a, 'r> =
    RequestBuilder<'a, 'r, SendTriggeredCanvasesRequest, SendTriggeredCanvasesResponse>;

/// Builder for [`Messaging::create_send_ids`].
pub type CreateSendIds<'a, 'r> =
    RequestBuilder<'a, 'r, CreateSendIdsRequest, CreateSendIdsResponse>;

/// Builder for [`Messaging::send_transactional_message`].
pub type SendTransactionalMessage<'a, 'r> =
    RequestBuilder<'a, 'r, SendTransactionalMessageRequest, SendTransactionalMessageResponse>;
