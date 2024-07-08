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
            let text_lines = Slack.issue_slack_message_text_lines(&issue, &body, &"".to_string());
            let message = Slack.construct_gh_issue_slack_message(&issue, &text_lines);
            let slash_command_response = SlashCommandResponse {
                blocks: message,
                response_type: "in_channel".to_string(),
            };

            let json = serde_json::json!(&slash_command_response);

            Response::from_json(&json)
        },
        Err(e) => Response::error(format!("{:?}", e), 500),
    }
}

pub async fn webhook(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let body: GithubWebhookRequest = req.json().await?;

    let prefix_text = format!("An issue was {}", body.action);
    let issue_string = format!("{}/{}#{}", body.repository.owner.login, body.repository.name, body.issue.number);

    let issue = body.issue;

    let text_lines = Slack.issue_slack_message_text_lines(&issue, &issue_string, &prefix_text);

    let message = Slack.construct_gh_issue_slack_message(&issue, &text_lines);
    let slash_command_response = SlashCommandResponse {
            blocks: message,
            response_type: "in_channel".to_string(),
    };

    let slack_webhook_url = ctx.secret("SLACK_WEBHOOK_URL")?.to_string();

    let result = Slack
        .send_issue_slack_message(&slack_webhook_url, &slash_command_response)
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

        let text_lines = Slack.issue_slack_message_text_lines(&issue, &body, &"".to_string());
        let message = Slack.construct_gh_issue_slack_message(&issue, &text_lines);
        let result = SlashCommandResponse {
            blocks: message,
            response_type: "in_channel".to_string(),
        };

        let slack_message = r#"{"blocks":[{"type":"section","text":{"type":"mrkdwn","text":"\n`*test - <https://github.com/cloudflare/wrangler-legacy/issues/1|token=gIkuvaNzQIHg97ATvDxqgjtO&team_id=T0001&team_domain=example&enterprise_id=E0001&enterprise_name=Globular%2520Construct%2520Inc&channel_id=C2147483705&channel_name=test&user_id=U2147483697&user_name=Steve&command=%2Fissue&text=cloudflare%2Fwrangler%231&response_url=https%3A%2F%2Fhooks.slack.com%2Fcommands%2F1234%2F5678&trigger_id=13345224609.738474920.8088930838d88f008e0root@d1cdcb320e3f>*`\nbody\n`*open* - Created by <https://github.com/signalnerve|test> on 2024-07-07 20:09:31`"},"accessory":{"type":"image","image_url":"https://github.com/images/error/octocat_happy.gif","alt_text":"test"},"response_type":"in_channel"}]}"#;

        let json = serde_json::to_string(&result).unwrap();

        assert_eq!(json, slack_message);
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

        let prefix_text = format!("An issue was {}", body.action);
        let issue_string = format!("{}/{}#{}", body.repository.owner.login, body.repository.name, body.issue.number);
        let issue = body.issue;
        let text_lines = Slack.issue_slack_message_text_lines(&issue, &issue_string, &prefix_text);
        let message = Slack.construct_gh_issue_slack_message(&issue, &text_lines);

        let result = vec![SlackMessage {
            r#type: "section".to_string(),
            text: Text {
                r#type: "mrkdwn".to_string(),
                text: text_lines,
            },
            accessory: Accessory {
                r#type: "image".to_string(),
                image_url: issue.user.avatar_url,
                alt_text: issue.user.login,
            },
            //response_type: "in_channel".to_string(),
        }];

        assert_eq!(result, message);

    }
}
