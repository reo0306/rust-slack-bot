use worker::*;

use crate::adapter::{
    github::Github,
    slack::Slack,
};
use crate::domain::model::{
    routes::{
        SlashCommandRequest,
        GithubWebhookRequest,
    },
    slack::Message,
};

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
                Message {
                    blocks: Slack.construct_message(
                        &issue,
                        &Slack.text_lines(&issue, &body, "")
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

    let message = match (&body.issue, &body.pull_request) {
        (Some(issue), None) => Slack.create_message(issue, &body),
        (None, Some(pr)) => Slack.create_message(pr, &body),
        _ => return Response::error("Unknown payload", 400),
    };

    let result = Slack.send_message(
            &ctx.secret("SLACK_WEBHOOK_URL")?.to_string(),
            &message
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
    use crate::domain::model::{
        github::{Issue, User, PullRequest},
        routes::{GithubWebhookRequest, ReqOwner, ReqRepo},
        slack::{Accessory, Message, Blocks, Text},
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
            body: Some("body".to_string()),
            state: "open".to_string(),
            created_at: "2024-07-07T20:09:31Z".to_string(),
            number: 1,
            user: lookup_user,
        };

        let body = "token=gIkuvaNzQIHg97ATvDxqgjtO&team_id=T0001&team_domain=example&enterprise_id=E0001&enterprise_name=Globular%2520Construct%2520Inc&channel_id=C2147483705&channel_name=test&user_id=U2147483697&user_name=Steve&command=%2Fissue&text=cloudflare%2Fwrangler%231&response_url=https%3A%2F%2Fhooks.slack.com%2Fcommands%2F1234%2F5678&trigger_id=13345224609.738474920.8088930838d88f008e0root@d1cdcb320e3f".to_string();

        let result = Message {
            blocks: Slack.construct_message(
                &issue,
                &Slack.text_lines(&issue, &body, "")
            ),
            response_type: "in_channel".to_string(),
        };

        let slack_message: Message = serde_json::from_str(r#"{"blocks":[{"type":"section","text":{"type":"mrkdwn","text":"*test - <https://github.com/cloudflare/wrangler-legacy/issues/1|>*\nbody\n*open* - Created by <https://github.com/signalnerve|test> on 2024-07-07 20:09:31"},"accessory":{"type":"image","image_url":"https://github.com/images/error/octocat_happy.gif","alt_text":"test"}}],"response_type":"in_channel"}"#).unwrap();

        assert_eq!(serde_json::json!(&slack_message), serde_json::json!(&result));
    }

    #[test]
    fn test_issue() {
        let body = GithubWebhookRequest {
            action: "opened".to_string(),
            issue: Some(Issue {
                html_url: "https://github.com/cloudflare/wrangler-legacy/issues/1".to_string(),
                title: "test".to_string(),
                body: Some("body".to_string()),
                state: "open".to_string(),
                created_at: "2024-07-07T20:09:31Z".to_string(),
                number: 1,
                user: User {
                    html_url: "https://github.com/signalnerve".to_string(),
                    login: "test".to_string(),
                    avatar_url: "https://github.com/images/error/octocat_happy.gif".to_string(),
                }
            }),
            pull_request: None,
            repository: ReqRepo {
                name: "test_repo".to_string(),
                owner: ReqOwner {
                    login: "test".to_string(),
                }
            },
        };

        let issue = body.issue.clone().unwrap();

        let text_lines = Slack.text_lines(
            &issue,
            &format!("{}/{}#{}", body.repository.owner.login, body.repository.name, issue.number),
            &format!("An {} was {}", body.label(), body.action)
        );

        let result = Slack.construct_message(&issue, &text_lines);

        assert_eq!(vec![Blocks {
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
        }], result);

    }

    #[test]
    fn test_issue_is_body_none() {
        let body = GithubWebhookRequest {
            action: "opened".to_string(),
            issue: Some(Issue {
                html_url: "https://github.com/cloudflare/wrangler-legacy/issues/1".to_string(),
                title: "test".to_string(),
                body: None,
                state: "open".to_string(),
                created_at: "2024-07-07T20:09:31Z".to_string(),
                number: 1,
                user: User {
                    html_url: "https://github.com/signalnerve".to_string(),
                    login: "test".to_string(),
                    avatar_url: "https://github.com/images/error/octocat_happy.gif".to_string(),
                }
            }),
            pull_request: None,
            repository: ReqRepo {
                name: "test_repo".to_string(),
                owner: ReqOwner {
                    login: "test".to_string(),
                }
            },
        };

        let issue = body.issue.clone().unwrap();

        let text_lines = Slack.text_lines(
            &issue,
            &format!("{}/{}#{}", body.repository.owner.login, body.repository.name, issue.number),
            &format!("An {} was {}", body.label(), body.action)
        );

        let result = Slack.construct_message(&issue, &text_lines);

        assert_eq!(vec![Blocks {
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
        }], result);

    }

    #[test]
    fn test_pull_request() {
        let body = GithubWebhookRequest {
            action: "opened".to_string(),
            pull_request: Some(PullRequest {
                html_url: "https://github.com/reo0306/rust-todo-di-app/pull/1".to_string(),
                title: "test pull_request".to_string(),
                body: Some("pull_request body".to_string()),
                state: "open".to_string(),
                created_at: "2024-07-07T20:09:31Z".to_string(),
                number: 2,
                user: User {
                    html_url: "https://github.com/reo0306/".to_string(),
                    login: "test2".to_string(),
                    avatar_url: "https://github.com/images/error/octocat_happy.gif".to_string(),
                }
            }),
            issue: None,
            repository: ReqRepo {
                name: "test_repo2".to_string(),
                owner: ReqOwner {
                    login: "test2".to_string(),
                }
            },
        };

        let pull_request = body.pull_request.clone().unwrap();

        let text_lines = Slack.text_lines(
            &pull_request,
            &format!("{}/{}#{}", body.repository.owner.login, body.repository.name, pull_request.number),
            &format!("An {} was {}", body.label(), body.action)
        );

        let result = Slack.construct_message(&pull_request, &text_lines);

        assert_eq!(vec![Blocks {
            r#type: "section".to_string(),
            text: Text {
                r#type: "mrkdwn".to_string(),
                text: text_lines,
            },
            accessory: Accessory {
                r#type: "image".to_string(),
                image_url: pull_request.user.avatar_url,
                alt_text: pull_request.user.login,
            },
        }], result);

    }

    #[test]
    fn test_pull_request_is_body_none() {
        let body = GithubWebhookRequest {
            action: "opened".to_string(),
            pull_request: Some(PullRequest {
                html_url: "https://github.com/reo0306/rust-todo-di-app/pull/1".to_string(),
                title: "test pull_request".to_string(),
                body: None,
                state: "open".to_string(),
                created_at: "2024-07-07T20:09:31Z".to_string(),
                number: 2,
                user: User {
                    html_url: "https://github.com/reo0306/".to_string(),
                    login: "test2".to_string(),
                    avatar_url: "https://github.com/images/error/octocat_happy.gif".to_string(),
                }
            }),
            issue: None,
            repository: ReqRepo {
                name: "test_repo2".to_string(),
                owner: ReqOwner {
                    login: "test2".to_string(),
                }
            },
        };

        let pull_request = body.pull_request.clone().unwrap();

        let text_lines = Slack.text_lines(
            &pull_request,
            &format!("{}/{}#{}", body.repository.owner.login, body.repository.name, pull_request.number),
            &format!("An {} was {}", body.label(), body.action)
        );

        let result = Slack.construct_message(&pull_request, &text_lines);

        assert_eq!(vec![Blocks {
            r#type: "section".to_string(),
            text: Text {
                r#type: "mrkdwn".to_string(),
                text: text_lines,
            },
            accessory: Accessory {
                r#type: "image".to_string(),
                image_url: pull_request.user.avatar_url,
                alt_text: pull_request.user.login,
            },
        }], result);

    }
}
