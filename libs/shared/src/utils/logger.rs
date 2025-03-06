use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, FmtSubscriber};



pub fn init_logger(){
    let subscriber = tracing_subscriber::registry()
        .with(fmt::layer());
    tracing::subscriber::set_global_default(subscriber).ok();
}


// #[ctor::ctor]
// fn setup_log_before_tests() {
//     init_logger();
// }