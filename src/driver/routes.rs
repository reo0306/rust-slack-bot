use worker::*;

use crate::adapter::{
    github::Github,
    slack::Slack,
};
use crate::model::routes::{SlashCommandRequest, GithubWebhookRequest, SlashCommandResponse};

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
            let result = serde_json::json!(
                SlashCommandResponse {
                    blocks: Slack.construct_gh_issue_slack_message(
                        &issue,
                        &Slack.issue_slack_message_text_lines(&issue, &body, "")
                    ),
                    response_type: "in_channel".to_string(),
                }
            );

            Response::from_json(&result)
        },
        Err(e) => Response::error(format!("{:?}", e), 500),
    }
}

pub async fn webhook(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let body: GithubWebhookRequest = req.json().await?;

    let text_lines = Slack.issue_slack_message_text_lines(
        &body.issue,
        &format!("{}/{}#{}", body.repository.owner.login, body.repository.name, body.issue.number),
        &format!("An issue was {}", body.action)
    );

    let slash_command_response = SlashCommandResponse {
        blocks: Slack.construct_gh_issue_slack_message(&body.issue, &text_lines),
        response_type: "in_channel".to_string(),
    };

    let result = Slack
        .send_issue_slack_message(
            &ctx.secret("SLACK_WEBHOOK_URL")?.to_string(),
            &slash_command_response
        )
        .await
        .map_err(|e| worker::Error::RustError(format!("{}", e)));

    match result {
        Ok(_) => Response::ok("OK"),
        Err(e) => Response::error(format!("Unable to handle webhook, message:{:?}", e), 500),
    }
}

#[cfg(test)]
mod routes_test {
    use crate::adapter::slack::Slack;
    use crate::model::{
        github::{Issue, User},
        routes::{GithubWebhookRequest, ReqOwner, ReqRepo, SlashCommandResponse},
        slack::{Accessory, SlackMessage, Text},
    };

    #[test]
    fn test_lookup() {
        let lookup_user = User {
            html_url: "https://github.com/signalnerve".to_string(),
            login: "test".to_string(),
            avatar_url: "https://github.com/images/error/octocat_happy.gif".to_string(),
        };

        let issue = Issue {
            html_url: "https://github.com/cloudflare/wrangler-legacy/issues/1".to_string(),
            title: "test".to_string(),
            body: "body".to_string(),
            state: "open".to_string(),
            created_at: "2024-07-07T20:09:31Z".to_string(),
            number: 1,
            user: lookup_user,
        };

        let body = "token=gIkuvaNzQIHg97ATvDxqgjtO&team_id=T0001&team_domain=example&enterprise_id=E0001&enterprise_name=Globular%2520Construct%2520Inc&channel_id=C2147483705&channel_name=test&user_id=U2147483697&user_name=Steve&command=%2Fissue&text=cloudflare%2Fwrangler%231&response_url=https%3A%2F%2Fhooks.slack.com%2Fcommands%2F1234%2F5678&trigger_id=13345224609.738474920.8088930838d88f008e0root@d1cdcb320e3f".to_string();

        let result = SlashCommandResponse {
            blocks: Slack.construct_gh_issue_slack_message(
                &issue,
                &Slack.issue_slack_message_text_lines(&issue, &body, "")
            ),
            response_type: "in_channel".to_string(),
        };

        let slack_message: SlashCommandResponse = serde_json::from_str(r#"{"blocks":[{"type":"section","text":{"type":"mrkdwn","text":"\n*test - <https://github.com/cloudflare/wrangler-legacy/issues/1|>*\nbody\n*open* - Created by <https://github.com/signalnerve|test> on 2024-07-07 20:09:31"},"accessory":{"type":"image","image_url":"https://github.com/images/error/octocat_happy.gif","alt_text":"test"}}],"response_type":"in_channel"}"#).unwrap();

        assert_eq!(serde_json::json!(&slack_message), serde_json::json!(&result));
    }

    #[test]
    fn test_webhook() {
        let webhook_user = User {
            html_url: "https://github.com/signalnerve".to_string(),
            login: "test".to_string(),
            avatar_url: "https://github.com/images/error/octocat_happy.gif".to_string(),
        };

        let webhook_issue = Issue {
            html_url: "https://github.com/cloudflare/wrangler-legacy/issues/1".to_string(),
            title: "test".to_string(),
            body: "body".to_string(),
            state: "open".to_string(),
            created_at: "2024-07-07T20:09:31Z".to_string(),
            number: 1,
            user: webhook_user,
        };

        let req_owner = ReqOwner {
            login: "test".to_string(),
        };

        let repo = ReqRepo {
            name: "test_repo".to_string(),
            owner: req_owner,
        };

        let body = GithubWebhookRequest {
            action: "opened".to_string(),
            issue: webhook_issue,
            repository: repo,
        };

        let text_lines = Slack.issue_slack_message_text_lines(
            &body.issue,
            &format!("{}/{}#{}", body.repository.owner.login, body.repository.name, body.issue.number),
            &format!("An issue was {}", body.action)
        );
        let result = Slack.construct_gh_issue_slack_message(&body.issue, &text_lines);

        assert_eq!(vec![SlackMessage {
            r#type: "section".to_string(),
            text: Text {
                r#type: "mrkdwn".to_string(),
                text: text_lines,
            },
            accessory: Accessory {
                r#type: "image".to_string(),
                image_url: body.issue.user.avatar_url,
                alt_text: body.issue.user.login,
            },
        }], result);

    }
}
