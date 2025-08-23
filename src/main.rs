use axum::serve;
use clap::Parser;
use clap::Subcommand;
use dotenvy::dotenv;
use sea_orm::ConnectOptions;
use sea_orm::Database;
use sea_orm::DatabaseConnection;
use tokio::net::TcpListener;
use tokio::runtime;
use tokio::spawn;
use tracing::log::LevelFilter;

use crate::web::State;
use crate::web::router;

mod entity;
mod load;
mod web;

fn main() -> anyhow::Result<()> {
	dotenv()?;
	tracing_subscriber::fmt::init();
	runtime::Builder::new_multi_thread()
		.enable_all()
		.build()?
		.block_on(async_main())
}

#[derive(Debug, Clone, Parser)]
struct Cli {
	#[command(subcommand)]
	command: Option<Command>,
	#[arg(long, env = "DATABASE_URL")]
	database_url: String,
}

#[derive(Debug, Clone, Default, Subcommand)]
enum Command {
	/// Start web server
	#[default]
	Server,
	/// Load random data into the database
	Load(load::Load),
}
async fn async_main() -> anyhow::Result<()> {
	let Cli {
		command,
		database_url,
	} = Cli::parse();
	let mut opts = ConnectOptions::new(&database_url);
	opts.sqlx_logging_level(LevelFilter::Debug);
	let db = Database::connect(opts).await?;

	let command = command.unwrap_or_default();

	match command {
		Command::Server => server(db).await,
		Command::Load(load) => load::load(db, load).await,
	}
}

async fn server(db: DatabaseConnection) -> anyhow::Result<()> {
	let state = State { db };
	let router = router().with_state(state);
	let listener = TcpListener::bind("0.0.0.0:12345").await?;
	eprintln!("listening {}", listener.local_addr()?);
	spawn(async move { serve(listener, router).await }).await??;

	Ok(())
}
