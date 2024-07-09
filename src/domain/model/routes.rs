use serde::{Serialize, Deserialize};

use crate::domain::model::{
    github::Issue,
    slack::SlackMessage
};

#[derive(Deserialize)]
pub struct SlashCommandRequest {
    pub text: String
}

#[derive(Serialize, Deserialize)]
pub struct SlashCommandResponse {
    pub blocks: Vec<SlackMessage>,
    pub response_type: String,
}

#[derive(Deserialize)]
pub struct GithubWebhookRequest {
    pub action: String,
    pub issue: Issue,
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

