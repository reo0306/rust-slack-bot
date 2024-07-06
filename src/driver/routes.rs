use worker::*;
//use serde::{Serialize, Deserialize};

use crate::adapter::{
    github::Github,
    slack::Slack,
};
use crate::model::routes::{SlashCommandResponse, SlashCommandRequest, GithubWebhookRequest};

/*#[derive(Deserialize)]
struct SlashCommandRequest {
    text: String
}

#[derive(Serialize)]
struct SlashCommandResponse {
    blocks: SlackMessage,
    response_type: String,
}

#[derive(Deserialize)]
struct GithubWebhookRequest {
    action: String,
    issue: Issue,
    repository: ReqRepo,
}

#[derive(Deserialize)]
struct ReqRepo {
    name: String,
    owner: ReqOwner,
}

#[derive(Deserialize)]
struct ReqOwner {
    login: String,
}*/

pub async fn lookup(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let body = req.text().await?;

    if body.is_empty() {
        return Response::error("Not found", 404);
    }

    let params: SlashCommandRequest = serde_qs::from_str(&body).unwrap();

    let issue = Github
        .fetch_issue(&params.text)
        .await
        .map_err(|e| worker::Error::RustError(format!("{}", e)));

    match issue {
        Ok(issue) => {
            let text_lines = Slack.issue_slack_message_text_lines(&issue, &body, &"".to_string());
            let message = Slack.construct_gh_issue_slack_message(&issue, &text_lines);
            let slash_command_response = SlashCommandResponse {
                blocks: message,
                response_type: "in_channel".to_string(),
            };

            Response::from_json(&slash_command_response)
        },
        Err(e) => Response::error(format!("{:?}", e), 500),
    }
}

pub async fn webhook(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let body: GithubWebhookRequest = req.json().await?;

    let prefix_text = format!("An issue was {}", body.action);
    let issue_string = format!("{}/{}#{}", body.repository.owner.login, body.repository.name, body.issue.number);

    let issue = body.issue;

    let text_lines = Slack.issue_slack_message_text_lines(&issue, &issue_string, &prefix_text);

    let message = Slack.construct_gh_issue_slack_message(&issue, &text_lines);

    let result = Slack
        .send_issue_slack_message(&message)
        .await
        .map_err(|e| worker::Error::RustError(format!("{}", e)));

    match result {
        Ok(_) => Response::ok("OK"),
        Err(e) => Response::error(format!("{}", e), 500),
    }
}
