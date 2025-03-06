

pub fn panic_hook(panic_info: &std::panic::PanicHookInfo) {
    let payload = panic_info.payload();
    #[allow(clippy::manual_map)]
    let payload = if let Some(s) = payload.downcast_ref::<&str>() {
        Some(&**s)
    } else if let Some(s) = payload.downcast_ref::<String>() {
        Some(s.as_str())
    } else {
        None
    };

    let location = panic_info.location().map(|l| l.to_string());

    tracing::error!(
        panic.payload = payload,
        panic.location = location,
        "A panic occurred: ",
    );
}