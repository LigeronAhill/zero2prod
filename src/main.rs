use tokio::net::TcpListener;
use zero2prod::{
    startup::{app, run},
    telemetry::{get_subscriber, init_subscriber},
    Result,
};
#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = get_subscriber("zero2prod", "info", std::io::stdout);
    init_subscriber(subscriber);
    let configuration = zero2prod::configuration::get_configuration().unwrap();
    let listener = TcpListener::bind(&configuration.app_addr()).await?;
    let db = zero2prod::Storage::init(configuration).await?;
    let app = app(db);
    run(listener, app).await?;
    Ok(())
}
