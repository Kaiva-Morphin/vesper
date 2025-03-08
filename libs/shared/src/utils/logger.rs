use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, Layer};



pub fn init_logger(){
    let subscriber = tracing_subscriber::registry()
        .with(fmt::layer().with_filter(LevelFilter::INFO));
    tracing::subscriber::set_global_default(subscriber).ok();
}


// #[ctor::ctor]
// fn setup_log_before_tests() {
//     init_logger();
// }