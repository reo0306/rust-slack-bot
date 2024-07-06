mod model;
mod adapter;
mod driver;

use worker::*;
use crate::driver::routes::{lookup, webhook};

/*
use serde::{Serialize, Deserialize};
use crate::github::Github;
use crate::slack::Slack;

use self::slack::SlackMessage;

#[derive(Deserialize)]
struct SlashCommandRequest {
    text: String }

#[derive(Serialize)]
struct SlashCommandResponse {
    blocks: SlackMessage,
    response_type: String,
}

#[derive(Deserialize)]
struct GithubWebhookRequest {
    action: String,
    issue: String,
    repository: String,
}*/

#[event(fetch, respond_with_errors)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .post_async("/lookup", lookup)
        .post_async("/webhook", webhook)
        .run(req, env)
        .await

    /*router
        .post_async("/lookup", |mut req, _ctx| async move {
            let body = req.text().await?;

            if body.is_empty() {
                return Response::error("Not found", 404);
            }

            let params: SlashCommandRequest = serde_qs::from_str(&body).unwrap();

            let issue = Github
                .fetch_issue(&params.text)
                .await
                .map(|issue| issue)
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

            //let text_lines = Slack.issue_slack_message_text_lines(&issue, &body, &"".to_string());

            //let message = Slack.construct_gh_issue_slack_message(&issue, &text_lines);

            //let slash_command_response = SlashCommandResponse {
                //blocks: message,
                //response_type: "in_channel".to_string(),
            //};

            //Response::from_json(&slash_command_response)
        })
        .post_async("/webhook", |mut req, _ctx| async move {
            let body: GithubWebhookRequest = req.json().await?;

            let prefix_text = format!("An issue was {}", body.action);
            let issue_string = format!("{}/{}#{}", body.repository.owner.login, body.repository.name, body.issue.number);

            let issue = body.issue;

            let text_lines = Slack.issue_slack_message_text_lines(&issue, &issue_string, &prefix_text);

            let message = Slack.construct_gh_issue_slack_message(&issue, &text_lines);

            Slack.send_issue_slack_message(&message);

            Response::text("ok")
        })
        .run(req, env)
        .await
*/
}

/*pub async fn lookup(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
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

    Slack.send_issue_slack_message(&message);

    Response::text("ok")
}*/
