use serde::{Serialize, Deserialize};

pub trait PayloadRepository {
    fn html_url(&self) -> &str;
    fn title(&self) -> &str;
    fn body(&self) -> Option<&str>;
    fn state(&self) -> &str;
    fn created_at(&self) -> &str;
    fn number(&self) -> u32;
    fn user(&self) -> &User;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Issue {
  pub html_url: String,
  pub title: String,
  pub body: Option<String>,
  pub state: String,
  pub created_at: String,
  pub number: u32,
  pub user: User,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PullRequest {
  pub html_url: String,
  pub title: String,
  pub body: Option<String>,
  pub state: String,
  pub created_at: String,
  pub number: u32,
  pub user: User,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
  pub html_url: String,
  pub login: String,
  pub avatar_url: String,
}

impl PayloadRepository for Issue {
    fn html_url(&self) -> &str {
        &self.html_url
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    fn state(&self) -> &str {
        &self.state
    }

    fn created_at(&self) -> &str {
        &self.created_at
    }

    fn number(&self) -> u32 {
        self.number
    }

    fn user(&self) -> &User {
        &self.user
    }
}

impl PayloadRepository for PullRequest {
    fn html_url(&self) -> &str {
        &self.html_url
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    fn state(&self) -> &str {
        &self.state
    }

    fn created_at(&self) -> &str {
        &self.created_at
    }

    fn number(&self) -> u32 {
        self.number
    }

    fn user(&self) -> &User {
        &self.user
    }
}
