use tracing::Level;
use tracing_subscriber::FmtSubscriber;




pub fn init_logger(){
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::WARN)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");
}


#[ctor::ctor]
fn setup_log_before_tests() {
    init_logger();
}