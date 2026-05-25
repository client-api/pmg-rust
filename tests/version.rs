// SC-01 — /version on PMG. PMG 9.x requires auth for /version, so the
// scenario goes through a ticket session (PMG has no token API).

mod common;

use clientapi_pmg::apis::{access_ticket_api, version_api};
use clientapi_pmg::models::AccessTicketCreateTicketRequest;
use common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sc_01_version_returns_expected_shape() {
    let creds = Credentials::from_env();
    let anon = creds.config_anonymous();

    let login = access_ticket_api::access_ticket_create_ticket(
        &anon,
        AccessTicketCreateTicketRequest::new(creds.password.clone(), creds.user.clone()),
    )
    .await
    .expect("login");
    let ticket = login.data.ticket.expect("ticket");
    let csrf = login.data.csrf_prevention_token.expect("csrf");
    let cfg = creds.config_with_ticket(&ticket, &csrf);

    let resp = version_api::version_version(&cfg)
        .await
        .expect("GET /version");

    assert!(
        resp.data.version.starts_with('9'),
        "expected version 9.x, got {:?}",
        resp.data.version
    );
    assert!(
        !resp.data.release.is_empty(),
        "release must be non-empty"
    );
}
