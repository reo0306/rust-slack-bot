use std::error::Error;
use chrono::NaiveDateTime;
use regex::Regex;

use crate::model::{
    github::Issue,
    slack::{SlackMessage, Text, Accessory},
    routes::SlashCommandResponse,
};

pub struct Slack;

impl Slack {
    pub fn issue_slack_message_text_lines(&self, issue: &Issue, body: &str, prefix_text: &String) -> String {
        let re = Regex::new(r"(?P<owner>[\w.-]*)\/(?P<repo>[\w.-]*)\#(?P<issue_number>\d*)").unwrap();

        let mut issue_string = String::new();

        for caps in re.captures_iter(body) {
            issue_string = format!("{}/{}#{}", &caps["owner"], &caps["repo"], &caps["issue_number"]);
        }

        let issue_link = format!("<{}|{}>", issue.html_url, issue_string);
	let user_link = format!("<{}|{}>", issue.user.html_url, issue.user.login);
        let created_date = NaiveDateTime::parse_from_str(&issue.created_at, "%Y-%m-%dT%H:%M:%SZ").unwrap();

        let title_uri = format!("*{} - {}*", issue.title, issue_link);
        let state_line = format!("*{}* - Created by {} on {}", issue.state, user_link, created_date);
        
	let text_lines = [
            prefix_text,
            title_uri.as_str(),
            &issue.body,
            state_line.as_str(),
        ];

        text_lines.join("\n")
    }

    pub fn construct_gh_issue_slack_message(&self, issue: &Issue, text_lines: &String) -> Vec<SlackMessage> {
        let avatar_url = &issue.user.avatar_url;
        let alt_text = &issue.user.login;

        vec![
            SlackMessage {
                r#type: "section".to_string(),
                text: Text {
                    r#type: "mrkdwn".to_string(),
                    text: text_lines.to_string(),
                },
                accessory: Accessory {
                    r#type: "image".to_string(),
                    image_url: avatar_url.to_string(),
                    alt_text: alt_text.to_string(),
                },
            }
        ]
    }

    pub async fn send_issue_slack_message(&self, slack_webhook_url: &str, slash_command_response: &SlashCommandResponse) -> Result<(), Box::<dyn Error>> {
        let client = reqwest::Client::new();

        let message = serde_json::json!(slash_command_response);

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
    use crate::model::github::{Issue, User};
    use crate::model::slack::{SlackMessage, Text, Accessory};

    #[test]
    fn test_issue_slack_message_text_lines() {
        let slack_user = User {
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
            user: slack_user,
        };

        let body = "token=gIkuvaNzQIHg97ATvDxqgjtO&team_id=T0001&team_domain=example&enterprise_id=E0001&enterprise_name=Globular%2520Construct%2520Inc&channel_id=C2147483705&channel_name=test&user_id=U2147483697&user_name=Steve&command=%2Fissue&text=cloudflare%2Fwrangler%231&response_url=https%3A%2F%2Fhooks.slack.com%2Fcommands%2F1234%2F5678&trigger_id=13345224609.738474920.8088930838d88f008e0root@d1cdcb320e3f".to_string();
        
        let result = Slack.issue_slack_message_text_lines(&issue, &body, &"".to_owned());

        let issue_link = format!("<{}|{}>", issue.html_url, body);
	let user_link = format!("<{}|{}>", issue.user.html_url, issue.user.login);
        let created_date = NaiveDateTime::parse_from_str(&issue.created_at, "%Y-%m-%dT%H:%M:%SZ").unwrap();
        let title_uri = format!("`*{} - {}*`", issue.title, issue_link);
        let state_line = format!("`*{}* - Created by {} on {}`", issue.state, user_link, created_date);
	let text_lines = [
            "",
            title_uri.as_str(),
            &issue.body,
            state_line.as_str(),
        ];

        assert_eq!(result, text_lines.join("\n"));
    }

    #[test]
    fn test_construct_gh_issue_slack_message() {
        let slack_user = User {
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
            user: slack_user,
        };

        let body = "token=gIkuvaNzQIHg97ATvDxqgjtO&team_id=T0001&team_domain=example&enterprise_id=E0001&enterprise_name=Globular%2520Construct%2520Inc&channel_id=C2147483705&channel_name=test&user_id=U2147483697&user_name=Steve&command=%2Fissue&text=cloudflare%2Fwrangler%231&response_url=https%3A%2F%2Fhooks.slack.com%2Fcommands%2F1234%2F5678&trigger_id=13345224609.738474920.8088930838d88f008e0root@d1cdcb320e3f".to_string();
        
        let text_lines = Slack.issue_slack_message_text_lines(&issue, &body, &"".to_owned());
        let result = Slack.construct_gh_issue_slack_message(&issue, &text_lines);

        let avatar_url = &issue.user.avatar_url;
        let alt_text = &issue.user.login;

        let slack_message = vec![SlackMessage {
            r#type: "section".to_string(),
            text: Text {
                r#type: "mrkdwn".to_string(),
                text: text_lines.to_string(),
            },
            accessory: Accessory {
                r#type: "image".to_string(),
                image_url: avatar_url.to_string(),
                alt_text: alt_text.to_string(),
            },
        }];

        assert_eq!(result, slack_message);
    }
}
