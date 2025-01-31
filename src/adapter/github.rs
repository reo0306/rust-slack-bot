use std::error::Error;
use regex::Regex;

use crate::domain::model::github::Issue;

pub struct Github;

impl Github {

    pub async fn fetch_issue(&self, text: &str) -> Result<Issue, Box<dyn Error>> {
        let re = Regex::new(r"(?P<owner>[\w.-]*)\/(?P<repo>[\w.-]*)\#(?P<issue_number>\d*)").unwrap();

        let mut url = String::new();

        for caps in re.captures_iter(text) {
            url = format!("https://api.github.com/repos/{}/{}/issues/{}", &caps["owner"], &caps["repo"], &caps["issue_number"]);
        }


        let client = reqwest::Client::builder()
            .user_agent("simple-worker-slack-bot")
            .build()?;

        let res = client
            .get(url)
            .send()
            .await?;

        let issue = res.json::<Issue>()
            .await?;

        Ok(issue)
    }
}
