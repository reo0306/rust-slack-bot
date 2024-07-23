use std::error::Error;
use chrono::NaiveDateTime;
use regex::Regex;

use crate::domain::model::{
    github::PayloadRepository,
    routes::GithubWebhookRequest,
    slack::{Accessory, Blocks, Message, Text, TextLine},
};

pub struct Slack;

impl Slack {
    pub fn create_message<T: PayloadRepository>(&self, payload: &T, body: &GithubWebhookRequest) -> Message {
        let text_lines = self.text_lines(
            payload,
            &format!("{}/{}#{}", body.repository.owner.login, body.repository.name, payload.number()),
            &format!("An issue was {}", body.action)
        );

        Message {
            blocks: self.construct_message(payload, &text_lines),
            response_type: "in_channel".to_string(),
        }
    }

    pub fn text_lines<T: PayloadRepository>(&self, payload: &T, body: &str, prefix_text: &str) -> String {
        let re = Regex::new(r"(?P<owner>[\w.-]*)\/(?P<repo>[\w.-]*)\#(?P<issue_number>\d*)").unwrap();

        let mut to_string = String::new();

        for caps in re.captures_iter(body) {
            to_string = format!("{}/{}#{}", &caps["owner"], &caps["repo"], &caps["issue_number"]);
        }

        let text_line = TextLine {
            title: format!("*{} - <{}|{}>*", payload.title(), payload.html_url(), to_string),
            state: format!(
                "*{}* - Created by <{}|{}> on {}",
                payload.state(),
                payload.user().html_url,
                payload.user().login,
                NaiveDateTime::parse_from_str(payload.created_at(), "%Y-%m-%dT%H:%M:%SZ").unwrap(),
            ),
        };

	[
            prefix_text,
            text_line.title.as_str(),
            payload.body(),
            text_line.state.as_str(),
        ].join("\n")
    }

    pub fn construct_message<T: PayloadRepository>(&self, payload: &T, text_lines: &String) -> Vec<Blocks> {
        vec![
            Blocks {
                r#type: "section".to_string(),
                text: Text {
                    r#type: "mrkdwn".to_string(),
                    text: text_lines.to_string(),
                },
                accessory: Accessory {
                    r#type: "image".to_string(),
                    image_url: payload.user().avatar_url.to_string(),
                    alt_text: payload.user().login.to_string(),
                },
            }
        ]
    }

    pub async fn send_message(&self, slack_webhook_url: &str, message : &Message) -> Result<(), Box::<dyn Error>> {
        let client = reqwest::Client::new();

        let message = serde_json::json!(message);

        client
            .post(slack_webhook_url)
            .json(&message)
            .send()
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod message_test {
    use super::*;
    use crate::domain::model::{
        github::{Issue, PullRequest, User},
        slack::{Blocks, Text, Accessory}
    };

    #[test]
    fn test_issue_text_lines() {
        let issue = Issue {
            html_url: "https://github.com/cloudflare/wrangler-legacy/issues/1".to_string(),
            title: "test".to_string(),
            body: "body".to_string(),
            state: "open".to_string(),
            created_at: "2024-07-07T20:09:31Z".to_string(),
            number: 1,
            user: User {
                html_url: "https://github.com/signalnerve".to_string(),
                login: "test".to_string(),
                avatar_url: "https://github.com/images/error/octocat_happy.gif".to_string(),
            }
        };

        assert_eq!(
            "\n*test - <https://github.com/cloudflare/wrangler-legacy/issues/1|>*\nbody\n*open* - Created by <https://github.com/signalnerve|test> on 2024-07-07 20:09:31".to_string(),
            Slack.text_lines(
                &issue,
                "token=gIkuvaNzQIHg97ATvDxqgjtO&team_id=T0001&team_domain=example&enterprise_id=E0001&enterprise_name=Globular%2520Construct%2520Inc&channel_id=C2147483705&channel_name=test&user_id=U2147483697&user_name=Steve&command=%2Fissue&text=cloudflare%2Fwrangler%231&response_url=https%3A%2F%2Fhooks.slack.com%2Fcommands%2F1234%2F5678&trigger_id=13345224609.738474920.8088930838d88f008e0root@d1cdcb320e3f",
                ""
            )
        );
    }

    #[test]
    fn test_pull_request_text_lines() {
        let pull_request = PullRequest {
            html_url: "https://github.com/reo0306/rust-todo-di-app/pull/1".to_string(),
            title: "test pull_request".to_string(),
            body: "body pull_request".to_string(),
            state: "open".to_string(),
            created_at: "2024-07-07T20:09:31Z".to_string(),
            number: 2,
            user: User {
                html_url: "https://github.com/reo0306".to_string(),
                login: "test2".to_string(),
                avatar_url: "https://github.com/images/error/octocat_happy.gif".to_string(),
            }
        };

        assert_eq!(
            "\n*test pull_request - <https://github.com/reo0306/rust-todo-di-app/pull/1|>*\nbody pull_request\n*open* - Created by <https://github.com/reo0306|test2> on 2024-07-07 20:09:31".to_string(),
            Slack.text_lines(
                &pull_request,
                "token=gIkuvaNzQIHg97ATvDxqgjtO&team_id=T0001&team_domain=example&enterprise_id=E0001&enterprise_name=Globular%2520Construct%2520Inc&channel_id=C2147483705&channel_name=test&user_id=U2147483697&user_name=Steve&command=%2Fissue&text=cloudflare%2Fwrangler%231&response_url=https%3A%2F%2Fhooks.slack.com%2Fcommands%2F1234%2F5678&trigger_id=13345224609.738474920.8088930838d88f008e0root@d1cdcb320e3f",
                ""
            )
        );
    }

    #[test]
    fn test_construct_message() {
        let issue = Issue {
            html_url: "https://github.com/cloudflare/wrangler-legacy/issues/1".to_string(),
            title: "test".to_string(),
            body: "body".to_string(),
            state: "open".to_string(),
            created_at: "2024-07-07T20:09:31Z".to_string(),
            number: 1,
            user: User {
                html_url: "https://github.com/signalnerve".to_string(),
                login: "test".to_string(),
                avatar_url: "https://github.com/images/error/octocat_happy.gif".to_string(),
            }
        };

        let text_lines = Slack.text_lines(
            &issue,
            "token=gIkuvaNzQIHg97ATvDxqgjtO&team_id=T0001&team_domain=example&enterprise_id=E0001&enterprise_name=Globular%2520Construct%2520Inc&channel_id=C2147483705&channel_name=test&user_id=U2147483697&user_name=Steve&command=%2Fissue&text=cloudflare%2Fwrangler%231&response_url=https%3A%2F%2Fhooks.slack.com%2Fcommands%2F1234%2F5678&trigger_id=13345224609.738474920.8088930838d88f008e0root@d1cdcb320e3f",
            ""
        );

        assert_eq!(
            vec![
                Blocks {
                    r#type: "section".to_string(),
                    text: Text {
                        r#type: "mrkdwn".to_string(),
                        text: text_lines.to_string(),
                    },
                    accessory: Accessory {
                        r#type: "image".to_string(),
                        image_url: issue.user.avatar_url.to_string(),
                        alt_text: issue.user.login.to_string(),
                    },
                }
            ],
            Slack.construct_message(&issue, &text_lines)
        );
    }
}
