use axum::Router;
use axum::extract::Path;
use axum::response::Redirect;
use axum::routing::get;
use axum::routing::put;
use maud::DOCTYPE;
use maud::Markup;
use maud::html;
use sea_orm::DatabaseConnection;
use tower_http::services::ServeDir;
use uuid::Uuid;

mod notes;
mod users;

#[cfg(test)]
mod test;

#[derive(Debug, Clone)]
pub struct State {
	pub db: DatabaseConnection,
}

pub fn router() -> Router<State> {
	let serve_dir = ServeDir::new("./static");
	Router::new()
		.route("/", get(Redirect::permanent("/users")))
		.route("/users", get(users::h_users))
		.route(
			"/users/{user_id}/",
			get(|Path(user_id): Path<Uuid>| async move {
				Redirect::permanent(&format!("/users/{user_id}"))
			}),
		)
		.route(
			"/users/{user_id}",
			get(|Path(user_id): Path<Uuid>| async move {
				Redirect::permanent(&format!("/users/{user_id}/notes"))
			}),
		)
		.route(
			"/users/{user}/notes",
			get(notes::h_notes).post(notes::h_note_new_post),
		)
		.route(
			"/users/{user}/notes/{user}",
			get(notes::h_note)
				.put(notes::h_note_edit_put)
				.delete(notes::h_note_delete),
		)
		.route("/users/{user}/notes/{note}/edit", get(notes::h_note_edit))
		.route(
			"/users/{user}/notes/{note}/toggle",
			put(notes::h_note_toggle),
		)
		.route("/users/{user}/notes/new", get(notes::h_note_new))
		.nest_service("/static", serve_dir)
}

pub fn base(title: &str, scripts: &[&str], main: &Markup) -> Markup {
	html!(
		(DOCTYPE)
		html lang="en" {
			head {
				title { (title) }
				meta charset="utf-8";
				meta name="viewport" content="width=device-width, initial-scale=1";
				link rel="stylesheet" href="/static/pico.css";
				link rel="stylesheet" href="/static/style.css";
				link rel="icon" type="image/x-icon" href="/static/favicon.ico";
				@for script in scripts {
					script src={"/static/"(script)} {}
				}
			}
			body {
				main {
					nav {
						ul {
							li {
								a href="/users" { "Home" }
							}
						}
					}
					(main)
				}
			}
		}
	)
}
