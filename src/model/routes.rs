use serde::{Serialize, Deserialize};

use crate::model::github::Issue;
use crate::model::slack::SlackMessage;

#[derive(Deserialize)]
pub struct SlashCommandRequest {
    pub text: String
}

#[derive(Serialize)]
pub struct SlashCommandResponse {
    pub blocks: SlackMessage,
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

