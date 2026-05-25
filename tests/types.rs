// SC-50, SC-51 — type edge cases on PMG.

mod common;

use clientapi_pmg::apis::{access_ticket_api, access_users_api};
use clientapi_pmg::apis::configuration::Configuration;
use clientapi_pmg::models::AccessTicketCreateTicketRequest;
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
async fn sc_50_bigint_fields_deserialize_as_i64() {
    let creds = Credentials::from_env();
    let cfg = admin_ticket_cfg(&creds).await;

    let resp = access_users_api::access_users_get_users(&cfg)
        .await
        .expect("list users");
    for u in &resp.data {
        let _: Option<i64> = u.tfa_locked_until;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sc_51_nullable_fields_decode_as_option_none() {
    let creds = Credentials::from_env();
    let cfg = admin_ticket_cfg(&creds).await;

    let resp = access_users_api::access_users_get_users(&cfg)
        .await
        .expect("list users");
    for u in &resp.data {
        let _: Option<String> = u.comment.clone();
        if let Some(c) = &u.comment {
            assert_ne!(c, "null", "string \"null\" leaking through Option layer");
        }
    }
}
