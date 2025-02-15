use omnitron_api::common::SessionExt;
use omnitron_gate_common::Secret;
use omnitron_gate_core::{authorize_ticket, consume_ticket, Services};
use poem::session::Session;
use poem::web::{Data, FromRequest};
use poem::{Endpoint, Middleware, Request};
use serde::Deserialize;

pub struct TicketMiddleware {}

impl TicketMiddleware {
  pub fn new() -> Self {
    TicketMiddleware {}
  }
}

pub struct TicketMiddlewareEndpoint<E: Endpoint> {
  inner: E,
}

impl<E: Endpoint> Middleware<E> for TicketMiddleware {
  type Output = TicketMiddlewareEndpoint<E>;

  fn transform(&self, inner: E) -> Self::Output {
    TicketMiddlewareEndpoint { inner }
  }
}

#[derive(Deserialize)]
struct QueryParams {
  #[serde(rename = "omnitron-ticket")]
  ticket: Option<String>,
}

impl<E: Endpoint> Endpoint for TicketMiddlewareEndpoint<E> {
  type Output = E::Output;

  async fn call(&self, req: Request) -> poem::Result<Self::Output> {
    let mut session_is_temporary = false;
    let session = <&Session>::from_request_without_body(&req).await?;
    let session = session.clone();

    {
      let params: QueryParams = req.params()?;

      let mut ticket_value = None;
      if let Some(t) = params.ticket {
        ticket_value = Some(t);
      }
      for h in req.headers().get_all(http::header::AUTHORIZATION) {
        let header_value = h.to_str().unwrap_or("").to_string();
        if let Some((token_type, token_value)) = header_value.split_once(' ') {
          if &token_type.to_lowercase() == "omnitron" {
            ticket_value = Some(token_value.to_string());
            session_is_temporary = true;
          }
        }
      }

      if let Some(ticket) = ticket_value {
        let services = Data::<&Services>::from_request_without_body(&req).await?;

        if let Some(ticket_model) = {
          let ticket = Secret::new(ticket);
          if let Some(res) = authorize_ticket(&services.db, &ticket).await? {
            consume_ticket(&services.db, &res.id).await?;
            Some(res)
          } else {
            None
          }
        } {
          session.set_auth(omnitron_api::common::SessionAuthorization::Ticket {
            username: ticket_model.username,
            target_name: ticket_model.target,
          });
        }
      }
    }

    let resp = self.inner.call(req).await;

    if session_is_temporary {
      session.clear();
    }

    resp
  }
}
