// SC-10, SC-11, SC-14 — authentication on PMG.
//
// SC-12 / SC-13 (token auth) don't apply: PMG 9.x has no token API.
// The credentials sentinel `(unsupported-by-pmg)` from proxmox-docker
// signals this fact and the suite avoids invoking token paths at all.

mod common;

use clientapi_pmg::apis::{access_ticket_api, access_users_api, version_api, Error};
use clientapi_pmg::models::{
    AccessTicketCreateTicketRequest, AccessUsersCreateUsersRequest, PmgRoleEnum,
};
use common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sc_10_ticket_auth_returns_ticket_and_csrf() {
    let creds = Credentials::from_env();
    let cfg = creds.config_anonymous();

    let req = AccessTicketCreateTicketRequest::new(creds.password.clone(), creds.user.clone());
    let resp = access_ticket_api::access_ticket_create_ticket(&cfg, req)
        .await
        .expect("POST /access/ticket");

    let ticket = resp.data.ticket.as_deref().expect("ticket field present");
    let csrf = resp
        .data
        .csrf_prevention_token
        .as_deref()
        .expect("csrf_prevention_token field present");

    assert!(ticket.starts_with("PMG:"), "ticket prefix: {ticket}");
    assert!(!csrf.is_empty(), "csrf must be non-empty");

    // Sanity: the ticket session reaches an authenticated endpoint.
    let ticket_cfg = creds.config_with_ticket(ticket, csrf);
    version_api::version_version(&ticket_cfg)
        .await
        .expect("authenticated /version with ticket");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sc_11_ticket_auth_rejects_bad_password() {
    let creds = Credentials::from_env();
    let cfg = creds.config_anonymous();

    let req = AccessTicketCreateTicketRequest::new(
        "definitely-not-the-password".to_string(),
        creds.user.clone(),
    );
    let err = access_ticket_api::access_ticket_create_ticket(&cfg, req)
        .await
        .expect_err("bad password must fail");
    assert_status(&err, 401);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sc_14_state_change_without_csrf_returns_401() {
    let creds = Credentials::from_env();

    let login_req = AccessTicketCreateTicketRequest::new(
        creds.password.clone(),
        creds.user.clone(),
    );
    let login = access_ticket_api::access_ticket_create_ticket(
        &creds.config_anonymous(),
        login_req,
    )
    .await
    .expect("login");
    let ticket = login.data.ticket.as_deref().expect("ticket");
    let csrf = login.data.csrf_prevention_token.as_deref().expect("csrf");

    let admin_cfg = creds.config_with_ticket(ticket, csrf);
    let _ = access_users_api::access_users_delete_users(&admin_cfg, "e2e-csrf-probe@pmg").await;

    let cfg = creds.config_with_ticket_no_csrf(ticket);

    let req = AccessUsersCreateUsersRequest::new(
        PmgRoleEnum::Audit,
        "e2e-csrf-probe@pmg".to_string(),
    );
    let err = access_users_api::access_users_create_users(&cfg, req)
        .await
        .expect_err("create_users without CSRF must fail");
    assert_status(&err, 401);
}

fn assert_status<T: std::fmt::Debug>(err: &Error<T>, expected: u16) {
    match err {
        Error::ResponseError(rc) => assert_eq!(
            rc.status.as_u16(),
            expected,
            "expected HTTP {expected}, got {} (body: {})",
            rc.status,
            rc.content
        ),
        other => panic!("expected ResponseError({expected}), got {other:?}"),
    }
}
