use dotenv::dotenv;
use proto::admin_server::AdminServer;
use proto::finance_control_server::FinanceControlServer;
use sqlx::postgres::PgPool;
use std::env;
use std::{net::SocketAddr, sync::Arc};
use tonic::transport::Server;

use handlers::admin::AdminService;
use handlers::finance_control::FinanceControlService;

pub mod handlers;
pub mod models;

mod proto {
    tonic::include_proto!("finance_control");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("proto_descriptor");
}

type State = Arc<tokio::sync::RwLock<u64>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let db_url: &str = &env::var("DATABASE_URL").unwrap();
    let pool = PgPool::connect(db_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations").run(&pool).await?;

    let addr: SocketAddr = "[::1]:50051".parse()?;

    let state = State::default();

    let finance = FinanceControlService {
        state: state.clone(),
        db_pool: pool,
    };

    let admin = AdminService {
        state: state.clone(),
    };

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()?;

    Server::builder()
        .add_service(reflection)
        .add_service(AdminServer::new(admin))
        .add_service(FinanceControlServer::new(finance))
        .serve(addr)
        .await?;

    println!("Server running!");

    Ok(())
}
