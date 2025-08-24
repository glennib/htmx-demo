use axum::extract;
use maud::Markup;
use maud::html;
use num_format::Locale;
use num_format::ToFormattedString;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::FromQueryResult;
use sea_orm::QueryOrder;
use sea_orm::QuerySelect;
use uuid::Uuid;

use super::State;
use super::base;
use crate::entity::note;
use crate::entity::note::Entity as Note;
use crate::entity::users as user;
use crate::entity::users::Entity as User;

#[derive(FromQueryResult)]
struct Row {
	user_id: Uuid,
	name: String,
	notes_count: i64,
}

pub async fn h_users(extract::State(State { db }): extract::State<State>) -> Markup {
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
