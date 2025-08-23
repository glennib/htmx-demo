use clap::Args;
use clap::Subcommand;
use fake::Fake;
use rand::seq::SliceRandom;
use sea_orm::ActiveValue::Set;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;

use crate::entity::note;
use crate::entity::note::Entity as Note;
use crate::entity::users as user;
use crate::entity::users::Entity as User;

#[derive(Debug, Clone, Args)]
pub struct Load {
	#[command(subcommand)]
	load: LoadInner,
}

pub async fn load(db: DatabaseConnection, load: Load) -> anyhow::Result<()> {
	match load.load {
		LoadInner::Users { count } => users(db, count).await,
		LoadInner::Notes { count, max_users } => notes(db, count, max_users).await,
	}
}

async fn users(db: DatabaseConnection, count: u32) -> anyhow::Result<()> {
	let faker = fake::faker::name::en::Name();
	let users = (0..count).map(|_| {
		let name: String = faker.fake();
		println!("New user: {name}");
		user::ActiveModel {
			name: Set(name),
			..Default::default()
		}
	});
	let _res = User::insert_many(users).exec(&db).await?;
	println!("Loaded {count} new users");
	Ok(())
}

async fn notes(db: DatabaseConnection, count: u32, max_users: u32) -> anyhow::Result<()> {
	let title = fake::faker::lorem::en::Words(1..5);
	let body = fake::faker::lorem::en::Sentences(1..3);
	let mut users = User::find().all(&db).await?;
	users.shuffle(&mut rand::rng());
	users.truncate(usize::try_from(max_users).unwrap());

	let notes = users
		.iter()
		.map(|user::Model { user_id, name: _ }| {
			(0..count).map(|_| {
				let title = title.fake::<Vec<String>>().join(" ");
				let body = body.fake::<Vec<String>>().join(" ");
				note::ActiveModel {
					user_id: Set(*user_id),
					title: Set(title),
					body: Set(body),
					..Default::default()
				}
			})
		})
		.flatten();
	let notes: Vec<_> = notes.collect();
	let total = notes.len();
	let _res = Note::insert_many(notes).exec(&db).await?;
	println!(
		"Loaded {total} new notes ({count} notes each for {n_users} users)",
		n_users = users.len()
	);
	Ok(())
}

#[derive(Debug, Clone, Subcommand)]
enum LoadInner {
	/// Load random users into the database
	Users {
		/// Number of users to load
		#[arg(default_value = "1")]
		count: u32,
	},
	/// Load random notes into the database
	Notes {
		/// Number of notes to load per user
		#[arg(default_value = "5")]
		count: u32,
		/// Maximum number of users to load notes for (selected randomly)
		#[arg(long, default_value = "50")]
		max_users: u32,
	},
}
