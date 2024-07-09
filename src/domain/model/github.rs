use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Issue {
  pub html_url: String,
  pub title: String,
  pub body: String,
  pub state: String,
  pub created_at: String,
  pub number: u32,
  pub user: User,
}

#[derive(Serialize, Deserialize)]
pub struct User {
  pub html_url: String,
  pub login: String,
  pub avatar_url: String,
}

