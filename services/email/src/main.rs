use lettre::Address;
use shared::{env_config, utils::logger::init_logger};
use tracing::{error, Instrument};


mod subscriber;
mod email_templates;
mod mailer;

env_config!(
    ".env" => ENV = Env {
        EMAIL_PASSWORD : String,
        EMAIL_USERNAME : String,
        EMAIL_RELAY : String,
        EMAIL_SENDER : Address,

        EMAIL_SEND_NATS_EVENT : String,
        
        NATS_URL : String,
        NATS_PORT : String,
    }
);

#[tokio::main]
async fn main() {
    layers::make_unique_span!("email worker ", span); // todo: set global span
    init_logger();
    let r = subscriber::run_subscriber().instrument(span).await;
    if let Err(e) = r {
        error!(e);
    }
}