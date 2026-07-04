use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct DeploymentConfig {
    pub org: Option<String>,
    pub project: Option<String>,
    pub pat: Option<String>,
    pub bearer_token: Option<String>,
    pub api_pipeline_id: Option<u64>,
    pub frontend_pipeline_id: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DeploymentTarget {
    pub key: &'static str,
    pub label: &'static str,
    pub pipeline_id: Option<u64>,
    pub configured: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PipelineRunResponse {
    pub id: u64,
    pub name: Option<String>,
    pub state: Option<String>,
    pub result: Option<String>,
    #[serde(rename = "_links")]
    pub links: Option<PipelineRunLinks>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PipelineRunLinks {
    pub web: Option<PipelineRunLink>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PipelineRunLink {
    pub href: String,
}

#[derive(Debug)]
pub enum DeploymentError {
    MissingConfig(String),
    UnknownTarget(String),
    Request(String),
}

impl std::fmt::Display for DeploymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingConfig(message) => write!(f, "{}", message),
            Self::UnknownTarget(target) => write!(f, "Unknown deployment target: {}", target),
            Self::Request(message) => write!(f, "{}", message),
        }
    }
}

impl DeploymentConfig {
    pub fn from_env() -> Self {
        Self {
            org: env::var("AZURE_DEVOPS_ORG")
                .ok()
                .filter(|v| !v.trim().is_empty()),
            project: env::var("AZURE_DEVOPS_PROJECT")
                .ok()
                .filter(|v| !v.trim().is_empty()),
            pat: env::var("AZURE_DEVOPS_PAT")
                .ok()
                .filter(|v| !v.trim().is_empty()),
            bearer_token: env::var("AZURE_DEVOPS_BEARER_TOKEN")
                .ok()
                .filter(|v| !v.trim().is_empty()),
            api_pipeline_id: env::var("AZURE_PIPELINE_API_ID")
                .ok()
                .and_then(|v| v.parse().ok()),
            frontend_pipeline_id: env::var("AZURE_PIPELINE_FRONTEND_ID")
                .ok()
                .and_then(|v| v.parse().ok()),
        }
    }

    pub fn is_base_configured(&self) -> bool {
        self.org.is_some()
            && self.project.is_some()
            && (self.pat.is_some() || self.bearer_token.is_some())
    }

    pub fn is_all_configured(&self) -> bool {
        self.is_base_configured()
            && self.api_pipeline_id.is_some()
            && self.frontend_pipeline_id.is_some()
    }

    pub fn targets(&self) -> Vec<DeploymentTarget> {
        vec![
            DeploymentTarget {
                key: "api",
                label: "Workforce API",
                pipeline_id: self.api_pipeline_id,
                configured: self.is_base_configured() && self.api_pipeline_id.is_some(),
            },
            DeploymentTarget {
                key: "frontend",
                label: "Workforce frontend",
                pipeline_id: self.frontend_pipeline_id,
                configured: self.is_base_configured() && self.frontend_pipeline_id.is_some(),
            },
        ]
    }

    fn pipeline_id_for(&self, target: &str) -> Result<u64, DeploymentError> {
        match target {
            "api" => self.api_pipeline_id.ok_or_else(|| {
                DeploymentError::MissingConfig("AZURE_PIPELINE_API_ID is not configured.".to_string())
            }),
            "frontend" => self.frontend_pipeline_id.ok_or_else(|| {
                DeploymentError::MissingConfig(
                    "AZURE_PIPELINE_FRONTEND_ID is not configured.".to_string(),
                )
            }),
            other => Err(DeploymentError::UnknownTarget(other.to_string())),
        }
    }

    pub async fn trigger_pipeline(
        &self,
        target: &str,
        client: Arc<Client>,
    ) -> Result<PipelineRunResponse, DeploymentError> {
        let org = self.org.as_ref().ok_or_else(|| {
            DeploymentError::MissingConfig("AZURE_DEVOPS_ORG is not configured.".to_string())
        })?;
        let project = self.project.as_ref().ok_or_else(|| {
            DeploymentError::MissingConfig("AZURE_DEVOPS_PROJECT is not configured.".to_string())
        })?;
        let pipeline_id = self.pipeline_id_for(target)?;

        let url = format!(
            "https://dev.azure.com/{}/{}/_apis/pipelines/{}/runs?api-version=7.1-preview.1",
            org, project, pipeline_id
        );

        let mut request = client.post(url).json(&serde_json::json!({}));
        request = if let Some(pat) = self.pat.as_ref() {
            request.basic_auth("", Some(pat))
        } else if let Some(token) = self.bearer_token.as_ref() {
            request.bearer_auth(token)
        } else {
            return Err(DeploymentError::MissingConfig(
                "AZURE_DEVOPS_PAT or AZURE_DEVOPS_BEARER_TOKEN is not configured.".to_string(),
            ));
        };

        let response = request
            .send()
            .await
            .map_err(|e| {
                DeploymentError::Request(format!("Could not contact Azure DevOps: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "".to_string());
            return Err(DeploymentError::Request(format!(
                "Azure DevOps returned {}: {}",
                status, body
            )));
        }

        response
            .json::<PipelineRunResponse>()
            .await
            .map_err(|e| {
                DeploymentError::Request(format!("Could not read Azure DevOps response: {}", e))
            })
    }
}
