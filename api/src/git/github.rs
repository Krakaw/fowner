use crate::{FownerError, Project};
use awc::Client;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub struct Github {
    pub api_url: String,
    api_token: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct Pr {
    pub labels: Vec<Label>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Label {
    pub name: Option<String>,
}
const LABEL_REGEX: &'static str = "^[fF]:\\s*";

impl Github {
    pub fn new(api_url: String, api_token: Option<String>) -> Self {
        Self { api_url, api_token }
    }

    /// https://docs.github.com/en/rest/commits/commits#list-pull-requests-associated-with-a-commit
    pub async fn fetch_labels_for_commit(
        &self,
        commit_sha: &str,
    ) -> Result<Vec<String>, FownerError> {
        let url = format!("{}/commits/{}/pulls", self.api_url, commit_sha);
        let client = Client::default();
        let req = client
            .get(url)
            .insert_header(("User-Agent", "Fowner"))
            .insert_header(("Accept", "application/vnd.github+json"));
        let req = if let Some(api_token) = &self.api_token {
            req.insert_header(("Authorization", format!("token {}", api_token)))
        } else {
            req
        };

        let mut res = req.send().await?;
        let pull_request_data: Vec<Pr> = res.json().await?;
        let mut labels = vec![];
        let re = Regex::new(LABEL_REGEX).unwrap();
        for pr in pull_request_data {
            for label in pr.labels.iter().filter_map(|l| {
                let name = l.name.clone().unwrap_or_default().trim().to_string();
                if !re.is_match(name.as_str()) {
                    return None;
                }
                let name = re.replace(name.as_str(), "").to_string();
                if !name.is_empty() {
                    Some(name)
                } else {
                    None
                }
            }) {
                labels.push(label);
            }
        }
        Ok(labels)
    }
}

impl TryFrom<&Project> for Github {
    type Error = FownerError;

    fn try_from(project: &Project) -> Result<Self, Self::Error> {
        let api_url = project.get_github_api_url()?;
        let api_token = project.github_api_token.clone();
        Ok(Github::new(api_url, api_token))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[actix_web::test]
    async fn fetch_labels() {
        let github = Github::new(
            "https://api.github.com/repos/Krakaw/fowner".to_string(),
            None,
        );
        let sha = "a69560db7e8f23e371ed384203e55d6a031cb3dc";
        let labels = github.fetch_labels_for_commit(sha).await.unwrap();
        assert_eq!(labels, vec!["API".to_string()]);
    }

    #[test]
    fn label_regex() {
        let re = Regex::new(LABEL_REGEX).unwrap();
        assert!(re.is_match("F: Feature"));
        assert!(re.is_match("F:Feature"));
        assert!(re.is_match("F:      Feature"));
        assert!(re.is_match("f: Feature"));
        assert!(re.is_match("f:Feature"));
        assert!(re.is_match("f:     Feature"));
        assert!(!re.is_match("Feature"));

        assert_eq!(re.replace("F: Feature", ""), "Feature");
        assert_eq!(re.replace("F:      Feature", ""), "Feature");
        assert_eq!(re.replace("F:Feature", ""), "Feature");
        assert_eq!(re.replace("f: Feature", ""), "Feature");
        assert_eq!(re.replace("f:Feature", ""), "Feature");
        assert_eq!(re.replace("Feature", ""), "Feature");
    }
}
