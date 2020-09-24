use std::env;

use tracing::{error, info};
use env_logger;

use oxidizer::{DB, entity::IEntity};

mod db;
mod error;
mod configuration;
mod transport;

use db::*;
use configuration::*;
use transport::*;

#[tokio::main]
async fn main() -> Result<(), String> {
    let cfg = load_config();

    env::set_var("RUST_LOG", &cfg.loglevel);
    env_logger::init();

    info!("{:#?}", cfg);

    let db_logger = LoggingHandler {
        db: connect_db().await?,
    };

    let addr = format!("[::1]:{}", cfg.port).parse().unwrap();
    info!("Server listening on {}", addr);

    EventsServerBuilder::new()
        .add(Box::new(db_logger))
        .redirecting(&cfg.targets)
        .run(addr).await
        .map_err(|e| e.to_string())
}

struct LoggingHandler {
    db: DB,
}

#[async_trait::async_trait]
impl IncomingHandler for LoggingHandler {
    async fn handle_event(&self, event: Event) {
        let name = event.name;
        let mut loggged = Logging::for_event(&name);

        match loggged.save(&self.db).await {
            Ok(true) => info!(event = name.as_str(), "Event saved"),
            Ok(false) => error!(event = name.as_str(), "Failed to save event", ),
            Err(error) => error!(event = name.as_str(),
                                 error = format!("{:?}", error).as_str(),
                                 "Failed to save event log"),
        };
    }
}

#[test]
fn testing() {
    let cfg = Config {
        port: 8080,
        targets: vec![8081, 8082],
        loglevel: "info".to_string(),
    };

    confy::store_path("config_template", cfg).unwrap();
}
