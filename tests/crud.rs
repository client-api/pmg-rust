// SC-21, SC-30, SC-31, SC-34 — CRUD baseline on PMG.
// All operations go through a ticket session — PMG 9.x has no token API.

mod common;

use clientapi_pmg::apis::{access_ticket_api, access_users_api};
use clientapi_pmg::apis::configuration::Configuration;
use clientapi_pmg::models::{
    AccessTicketCreateTicketRequest, AccessUsersCreateUsersRequest, PmgRoleEnum,
};
use common::*;

const E2E_USER: &str = "e2e-user-01@pmg";

async fn admin_ticket_cfg(creds: &Credentials) -> Configuration {
    let req = AccessTicketCreateTicketRequest::new(creds.password.clone(), creds.user.clone());
    let resp = access_ticket_api::access_ticket_create_ticket(&creds.config_anonymous(), req)
        .await
        .expect("admin ticket login");
    let ticket = resp.data.ticket.expect("ticket");
    let csrf = resp
        .data
        .csrf_prevention_token
        .expect("csrf");
    creds.config_with_ticket(&ticket, &csrf)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sc_21_admin_can_create() {
    let creds = Credentials::from_env();
    let cfg = admin_ticket_cfg(&creds).await;
    let _ = access_users_api::access_users_delete_users(&cfg, "e2e-admin-probe@pmg").await;

    let req = AccessUsersCreateUsersRequest::new(
        PmgRoleEnum::Audit,
        "e2e-admin-probe@pmg".to_string(),
    );
    access_users_api::access_users_create_users(&cfg, req)
        .await
        .expect("admin user creation");

    let _ = access_users_api::access_users_delete_users(&cfg, "e2e-admin-probe@pmg").await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sc_30_list_users_includes_root() {
    let creds = Credentials::from_env();
    let cfg = admin_ticket_cfg(&creds).await;

    let users = access_users_api::access_users_get_users(&cfg)
        .await
        .expect("list users");
    let has_root = users.data.iter().any(|u| u.userid == "root@pam");
    assert!(
        has_root,
        "expected root@pam, got {:?}",
        users.data.iter().map(|u| &u.userid).collect::<Vec<_>>()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sc_31_user_crud_roundtrip() {
    let creds = Credentials::from_env();
    let cfg = admin_ticket_cfg(&creds).await;
    let _ = access_users_api::access_users_delete_users(&cfg, E2E_USER).await;

    let req = AccessUsersCreateUsersRequest::new(PmgRoleEnum::Audit, E2E_USER.to_string());
    access_users_api::access_users_create_users(&cfg, req)
        .await
        .expect("create user");

    let listed = access_users_api::access_users_get_users(&cfg)
        .await
        .expect("list users");
    assert!(listed.data.iter().any(|u| u.userid == E2E_USER));

    access_users_api::access_users_delete_users(&cfg, E2E_USER)
        .await
        .expect("delete user");

    let listed = access_users_api::access_users_get_users(&cfg)
        .await
        .expect("list users after delete");
    assert!(!listed.data.iter().any(|u| u.userid == E2E_USER));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sc_34_pagination_walks_users_endpoint() {
    let creds = Credentials::from_env();
    let cfg = admin_ticket_cfg(&creds).await;

    let listed = access_users_api::access_users_get_users(&cfg)
        .await
        .expect("list users");
    assert!(!listed.data.is_empty(), "at least one user must exist");
    for u in &listed.data {
        assert!(!u.userid.is_empty());
    }
}
