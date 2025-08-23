use std::env;

use axum::serve;
use dotenvy::dotenv;
use sea_orm::Database;
use tokio::net::TcpListener;
use tokio::runtime;

use crate::web::State;
use crate::web::router;

mod entity;
mod web;

fn main() -> anyhow::Result<()> {
	dotenv()?;
	tracing_subscriber::fmt::init();
	runtime::Builder::new_multi_thread()
		.enable_all()
		.build()?
		.block_on(async_main())
}

async fn async_main() -> anyhow::Result<()> {
	let db = Database::connect(env::var("DATABASE_URL")?).await?;
	let state = State { db };
	let router = router().with_state(state);
	let listener = TcpListener::bind("0.0.0.0:12345").await?;
	eprintln!("listening {}", listener.local_addr()?);
	serve(listener, router).await?;

	Ok(())
}
