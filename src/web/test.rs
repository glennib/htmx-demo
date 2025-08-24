use std::env;

use axum_test::TestServer;
use dotenvy::dotenv;
use sea_orm::Database;

use crate::web::State;

async fn server() -> TestServer {
	dotenv().unwrap();
	let db_url = env::var("DATABASE_URL").unwrap();
	let router = crate::web::router();
	let db = Database::connect(&db_url).await.unwrap();
	let router = router.with_state(State { db });
	TestServer::new(router).unwrap()
}

#[tokio::test]
async fn index() {
	let server = server().await;
	let response = server.get("/").await;
	assert!(response.status_code().is_redirection(), "{response:#?}");
	let users = server.get("/users").await;
	assert!(users.status_code().is_success(), "{users:#?}");
	let html = users.text();
	insta::assert_ron_snapshot!("users", html);
	let notes = server
		.get("/users/0198d832-fb78-797d-8ca0-4bc3615ea4ad/notes")
		.await;
	assert!(notes.status_code().is_success(), "{notes:#?}");
	let html = notes.text();
	insta::assert_ron_snapshot!("notes", html);
}
