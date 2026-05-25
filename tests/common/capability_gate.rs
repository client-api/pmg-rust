#[macro_export]
macro_rules! skip_if_no_network {
    () => {
        if std::env::var("PROXMOX_NO_NETWORK").as_deref() == Ok("1") {
            eprintln!("SKIP: PROXMOX_NO_NETWORK=1 (air-gapped runner)");
            return;
        }
    };
}
