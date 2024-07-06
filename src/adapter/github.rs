use std::error::Error;
use regex::Regex;
//use serde::{Serialize, Deserialize};

use crate::model::github::Issue;

pub struct Github;

/*#[derive(Serialize, Deserialize)]
pub struct Issue {
  pub html_url: String,
  pub title: String,
  pub body: String,
  pub state: String,
  pub created_at: String,
  pub number: u32,
  pub user: User,
}

#[derive(Serialize, Deserialize)]
pub struct User {
  pub html_url: String,
  pub login: String,
  pub avatar_url: String,
}*/

impl Github {

    pub async fn fetch_issue(&self, text: &str) -> Result<Issue, Box<dyn Error>> {
        // textをowner, repo, issue_numberで取得するための正規表現
        let re = Regex::new(r"(?P<owner>[\w.-]*)\/(?P<repo>[\w.-]*)\#(?P<issue_number>\d*)").unwrap();

        let mut url = String::new();

        for caps in re.captures_iter(text) {
            // githubリクエストURL作成
            url = format!("https://api.github.com/repos/{}/{}/issues/{}", &caps["owner"], &caps["repo"], &caps["issue_number"]);
        }


        let client = reqwest::Client::builder()
            .user_agent("simple-worker-slack-bot")
            .build()?;

        // githubのissue取得リクエスト
        let res = client
            .get(url)
            .send()
            .await?;

        let issue = res.json::<Issue>()
            .await?;

        Ok(issue)
    }
}
