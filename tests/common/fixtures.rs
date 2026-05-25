use clientapi_pmg::apis::access_users_api;
use clientapi_pmg::apis::configuration::Configuration;

pub const E2E_PREFIX: &str = "e2e-";

/// Best-effort sweep of e2e-* users. PMG has no ACL, no tokens, no
/// storages — users are the only product-side state.
pub async fn cleanup_e2e(cfg: &Configuration) {
    let Ok(resp) = access_users_api::access_users_get_users(cfg).await else {
        return;
    };
    for u in resp.data {
        if u.userid.starts_with(E2E_PREFIX) {
            let _ = access_users_api::access_users_delete_users(cfg, &u.userid).await;
        }
    }
}
