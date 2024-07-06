use std::error::Error;
use chrono::NaiveDateTime;

use crate::model::github::Issue;
use crate::model::slack::{SlackMessage, Text, Accessory};

pub struct Slack;

impl Slack {
    pub fn issue_slack_message_text_lines(&self, issue: &Issue, text: &String, prefix_text: &String) -> String {
        let issue_link = format!("<{}|{}>", issue.html_url, text);
	let user_link = format!("<{}|{}>", issue.user.html_url, issue.user.login);
        let created_date = NaiveDateTime::parse_from_str(&issue.created_at, "%Y/%m/%d").unwrap();

        let title_uri = format!("{} - {}", issue.title, issue_link);
        let state_line = format!("{} - Created by {} on {}", issue.state, user_link, created_date);
        
	let text_lines = [
            prefix_text,
            title_uri.as_str(),
            &issue.body,
            state_line.as_str(),
        ];

        text_lines.join("\n")
    }

    pub fn construct_gh_issue_slack_message(&self, issue: &Issue, text_lines: &String) -> SlackMessage {
        let avatar_url = &issue.user.avatar_url;
        let alt_text = &issue.user.login;

        let slack_message = SlackMessage {
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
        };

        slack_message
    }

    pub async fn send_issue_slack_message(&self, message: &SlackMessage) -> Result<(), Box::<dyn Error>> {
        let client = reqwest::Client::new();

        client
            .post("".to_string())
            .json(message)
            .send()
            .await?;

        Ok(())
    }
}
