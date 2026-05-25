// SC-41 — error envelope handling on PMG.
// SC-40 (unknown vmid) and SC-42 (privsep) don't apply on PMG.

mod common;

use clientapi_pmg::apis::configuration::Configuration;
use clientapi_pmg::apis::{access_ticket_api, access_users_api, Error};
use clientapi_pmg::models::{
    AccessTicketCreateTicketRequest, AccessUsersCreateUsersRequest, PmgRoleEnum,
};
use common::*;

async fn admin_ticket_cfg(creds: &Credentials) -> Configuration {
    let req = AccessTicketCreateTicketRequest::new(creds.password.clone(), creds.user.clone());
    let resp = access_ticket_api::access_ticket_create_ticket(&creds.config_anonymous(), req)
        .await
        .expect("admin ticket login");
    let ticket = resp.data.ticket.expect("ticket");
    let csrf = resp.data.csrf_prevention_token.expect("csrf");
    creds.config_with_ticket(&ticket, &csrf)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sc_41_invalid_input_returns_4xx() {
    let creds = Credentials::from_env();
    let cfg = admin_ticket_cfg(&creds).await;

    // Empty userid is rejected by the schema (userid is required, format
    // `name@realm`).
    let req = AccessUsersCreateUsersRequest::new(PmgRoleEnum::Audit, String::new());
    let err = access_users_api::access_users_create_users(&cfg, req)
        .await
        .expect_err("empty userid must fail");

    match err {
        Error::ResponseError(rc) => {
            assert!(
                rc.status.is_client_error(),
                "expected 4xx, got {} (body: {})",
                rc.status,
                rc.content
            );
        }
        other => panic!("expected ResponseError, got {other:?}"),
    }
}
