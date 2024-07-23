use serde::Deserialize;

use crate::domain::model::github::{Issue, PullRequest};

#[derive(Deserialize)]
pub struct SlashCommandRequest {
    pub text: String
}

#[derive(Deserialize)]
pub struct GithubWebhookRequest {
    pub action: String,
    pub issue: Option<Issue>,
    pub pull_request: Option<PullRequest>,
    pub repository: ReqRepo,
}

#[derive(Deserialize)]
pub struct ReqRepo {
    pub name: String,
    pub owner: ReqOwner,
}

#[derive(Deserialize)]
pub struct ReqOwner {
    pub login: String,
}

