use shared::utils::logger::init_logger;






mod profile;





#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();
    Ok(())
}
