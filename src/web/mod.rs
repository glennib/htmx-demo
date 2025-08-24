use axum::Form;
use axum::Router;
use axum::extract;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::get;
use axum::routing::put;
use chrono::Utc;
use maud::DOCTYPE;
use maud::Markup;
use maud::html;
use num_format::Locale;
use num_format::ToFormattedString;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::FromQueryResult;
use sea_orm::IntoActiveModel;
use sea_orm::ModelTrait;
use sea_orm::Order;
use sea_orm::QueryOrder;
use sea_orm::QuerySelect;
use sea_orm::Set;
use sea_orm::sea_query::NullOrdering;
use serde::Deserialize;
use tower_http::services::ServeDir;
use uuid::Uuid;

use crate::entity::note;
use crate::entity::note::Entity as Note;
use crate::entity::users as user;
use crate::entity::users::Entity as User;

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
		.route("/users", get(h_users))
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
		.route("/users/{user}/notes", get(h_notes).post(h_note_new_post))
		.route(
			"/users/{user}/notes/{user}",
			get(h_note).put(h_note_edit_put).delete(h_note_delete),
		)
		.route("/users/{user}/notes/{note}/edit", get(h_note_edit))
		.route("/users/{user}/notes/{note}/toggle", put(h_note_toggle))
		.route("/users/{user}/notes/new", get(h_note_new))
		.nest_service("/static", serve_dir)
}

async fn h_users(extract::State(State { db }): extract::State<State>) -> Markup {
	#[derive(FromQueryResult)]
	struct Row {
		user_id: Uuid,
		name: String,
		notes_count: i64,
	}

	let users = User::find()
		.column_as(note::Column::NoteId.count(), "notes_count")
		.left_join(Note)
		.group_by(user::Column::Name)
		.group_by(user::Column::UserId)
		.order_by_desc(note::Column::NoteId.count())
		.into_model::<Row>()
		.all(&db)
		.await
		.unwrap();

	let total_notes = users.iter().map(|user| user.notes_count).sum::<i64>();

	base(
		"Users -- TODO app",
		&["htmx.js", "reload.js"],
		&html!(
			h1 {
				"Users"
			}
			p {
				"Total notes: "(total_notes.to_formatted_string(&Locale::en))"."
				"Total users: "(users.len().to_formatted_string(&Locale::en))"."
			}
			table {
				thead {
					tr {
						th {
							"User ID"
						}
						th {
							"Name"
						}
						th {
						}
					}
				}
				tbody {
					@for user in users {
						tr {
							td {
								pre {
									(user.user_id)
								}
							}
							td {
								(user.name)
							}
							td {
								a href={"/users/"(user.user_id)"/notes"} {
									"View notes ("(user.notes_count.to_formatted_string(&Locale::en))")"
								}
							}
						}
					}
				}
			}
		),
	)
}

async fn h_notes(
	extract::State(State { db }): extract::State<State>,
	Path(user_id): Path<Uuid>,
) -> Markup {
	let (user, notes) = User::find_by_id(user_id)
		.find_with_related(Note)
		.order_by_with_nulls(note::Column::UpdatedAt, Order::Desc, NullOrdering::Last)
		.order_by_desc(note::Column::CreatedAt)
		.all(&db)
		.await
		.unwrap()
		.pop()
		.unwrap();
	let n_notes = notes.len();
	let user::Model { user_id: _, name } = user;
	base(
		&format!("{name} -- TODO app"),
		&["htmx.js"],
		&html!(
			h1 { "Notes for " (name) }
			p { "Total notes: " (n_notes) }
			table {
				thead {
					tr {
						th style="width: 25%;" {
							"Title"
						}
						th style="width: 25%;" {
							"Body"
						}
						th style="width: 25%;" {
							"Completed"
						}
						th style="width: 25%;" {

						}
					}
				}
				tbody
					hx-target="closest tr"
					hx-swap="outerHTML"
				{
					@for note in notes {
						(note_row_view(note))
					}
					(note_row_new(user_id))
				}
			}
		),
	)
}

fn note_row_new(user_id: Uuid) -> Markup {
	html!(
	tr {
		td colspan="4" {
			button
				hx-get={"/users/"(user_id)"/notes/new"}
			{
				"New"
			}
		}
	}
	)
}

#[derive(Deserialize)]
pub struct NoteEdit {
	title: String,
	body: String,
}

#[derive(Deserialize)]
pub struct NoteNew {
	title: String,
	body: String,
}

async fn h_note_edit(
	extract::State(State { db }): extract::State<State>,
	Path((user_id, note_id)): Path<(Uuid, Uuid)>,
) -> Markup {
	let (note, user) = Note::find_by_id(note_id)
		.find_also_related(User)
		.one(&db)
		.await
		.unwrap()
		.unwrap();
	let user = user.unwrap();
	assert_eq!(user.user_id, user_id, "note does not belong to user");
	note_row_edit(note)
}

async fn h_note_edit_put(
	extract::State(State { db }): extract::State<State>,
	Path((user_id, note_id)): Path<(Uuid, Uuid)>,
	Form(note_edit): Form<NoteEdit>,
) -> Markup {
	let (note, user) = Note::find_by_id(note_id)
		.find_also_related(User)
		.one(&db)
		.await
		.unwrap()
		.unwrap();
	let user = user.unwrap();
	assert_eq!(user.user_id, user_id, "note does not belong to user");
	let mut note = note.into_active_model();
	note.title = Set(note_edit.title);
	note.body = Set(note_edit.body);
	note.updated_at = Set(Some(Utc::now().fixed_offset()));
	let note = note.update(&db).await.unwrap();
	note_row_view(note)
}
async fn h_note(
	extract::State(State { db }): extract::State<State>,
	Path((user_id, note_id)): Path<(Uuid, Uuid)>,
) -> Markup {
	let (note, user) = Note::find_by_id(note_id)
		.find_also_related(User)
		.one(&db)
		.await
		.unwrap()
		.unwrap();
	let user = user.unwrap();
	assert_eq!(user.user_id, user_id, "note does not belong to user");
	note_row_view(note)
}

async fn h_note_new(Path(user_id): Path<Uuid>) -> Markup {
	note_row_new_form(user_id)
}

fn note_row_new_form(user_id: Uuid) -> Markup {
	html! (
		tr {
			td colspan="3" {
				input
					name="title"
					autofocus
					placeholder="note title"
					hx-post={"/users/"(user_id)"/notes"}
					hx-include="closest tr"
					hx-trigger="keyup[key=='Enter']";
				input
					name="body"
					placeholder="note body"
					hx-post={"/users/"(user_id)"/notes"}
					hx-include="closest tr"
					hx-trigger="keyup[key=='Enter']";
			}
			td {
				button
					hx-post={"/users/"(user_id)"/notes"}
					hx-include="closest tr"
				{
					"Save"
				}
			}
		}
	)
}

async fn h_note_new_post(
	extract::State(State { db }): extract::State<State>,
	Path(user_id): Path<Uuid>,
	Form(note_new): Form<NoteNew>,
) -> Markup {
	let NoteNew { title, body } = note_new;
	let note = note::ActiveModel {
		user_id: Set(user_id),
		title: Set(title),
		body: Set(body),
		..Default::default()
	};
	let note = note.insert(&db).await.unwrap();
	html!((note_row_view(note))(note_row_new(user_id)))
}

async fn h_note_toggle(
	extract::State(State { db }): extract::State<State>,
	Path((user_id, note_id)): Path<(Uuid, Uuid)>,
) -> Markup {
	let (note, user) = Note::find_by_id(note_id)
		.find_also_related(User)
		.one(&db)
		.await
		.unwrap()
		.unwrap();
	let user = user.unwrap();
	assert_eq!(user.user_id, user_id, "note does not belong to user");
	let is_done = note.is_done;
	let mut note = note.into_active_model();
	note.is_done = Set(!is_done);
	note.updated_at = Set(Some(Utc::now().fixed_offset()));
	let note = note.update(&db).await.unwrap();
	note_row_view(note)
}

async fn h_note_delete(
	extract::State(State { db }): extract::State<State>,
	Path((user_id, note_id)): Path<(Uuid, Uuid)>,
) -> StatusCode {
	let (note, user) = Note::find_by_id(note_id)
		.find_also_related(User)
		.one(&db)
		.await
		.unwrap()
		.unwrap();
	let user = user.unwrap();
	assert_eq!(user.user_id, user_id, "note does not belong to user");
	note.delete(&db).await.unwrap();
	StatusCode::OK
}

fn note_row_view(note: note::Model) -> Markup {
	let note::Model {
		note_id,
		user_id,
		created_at: _,
		updated_at: _,
		title,
		body,
		is_done,
	} = note;
	html!(tr {
		td { (title) }
		td { (body) }
		td {
			button
				hx-put={"/users/"(user_id)"/notes/"(note_id)"/toggle"}
			{
				@if is_done {
					"☑"
				} @else {
					"☐"
				}
			}
		}
		td {
			button
				hx-get={"/users/"(user_id)"/notes/"(note_id)"/edit"}
			{
				"Edit"
			}
			button
				hx-delete={"/users/"(user_id)"/notes/"(note_id)}
			{
				"Delete"
			}
		}
	})
}

fn note_row_edit(note: note::Model) -> Markup {
	let note::Model {
		note_id,
		user_id,
		created_at: _,
		updated_at: _,
		title,
		body,
		is_done,
	} = note;
	html!(
		tr {
			td {
				input
					name="title"
					hx-put={"/users/"(user_id)"/notes/"(note_id)}
					hx-include="closest tr"
					hx-trigger="keyup[key=='Enter']"
					value=(title);
			}
			td {
				input
					name="body"
					hx-put={"/users/"(user_id)"/notes/"(note_id)}
					hx-include="closest tr"
					hx-trigger="keyup[key=='Enter']"
					value=(body);
			}
			td {
				@if is_done {
					"☑"
				} @else {
					"☐"
				}
			}
			td {
				button
					hx-get={"/users/"(user_id)"/notes/"(note_id)}
				{
					"Cancel"
				}
				button
					hx-put={"/users/"(user_id)"/notes/"(note_id)}
					hx-include="closest tr"
				{
					"Save"
				}
			}
		}
	)
}

fn base(title: &str, scripts: &[&str], main: &Markup) -> Markup {
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
