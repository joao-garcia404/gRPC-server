use dotenv::dotenv;
use proto::admin_server::AdminServer;
use proto::finance_control_server::FinanceControlServer;
use sqlx::postgres::PgPool;
use std::env;
use std::{net::SocketAddr, sync::Arc};
use tokio::signal;
use tonic::transport::Server;

use handlers::admin::AdminService;
use handlers::finance_control::FinanceControlService;
use tracing::{info, warn, Tracing};

pub mod handlers;
pub mod models;
pub mod tracing;

mod proto {
    tonic::include_proto!("finance_control");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("proto_descriptor");
}

type State = Arc<tokio::sync::RwLock<u64>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    Tracing::init();

    let db_url: &str = &env::var("DATABASE_URL").expect("DATABASE_URL not present.");
    let pool = PgPool::connect(db_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations").run(&pool).await?;

    let addr: SocketAddr = "0.0.0.0:50051".parse().unwrap();

    let state = State::default();

    let finance = FinanceControlService {
        state: state.clone(),
        db_pool: Arc::new(pool),
    };

    let admin = AdminService {
        state: state.clone(),
    };

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()?;

    info!("Server running!");

    Server::builder()
        .add_service(reflection)
        .add_service(AdminServer::new(admin))
        .add_service(FinanceControlServer::new(finance))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install SIGINT handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    warn!("Shutdown signal received");
}
