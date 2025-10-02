use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct TleData {
    pub tle0: String,
    pub tle1: String,
    pub tle2: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobRequestDTO {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub tle: TleData,
    pub rx_frequency: f64,
    pub tx_frequency: f64,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub status: String,
    pub message: Option<String>,
}

pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
}

impl ApiClient {
    pub fn new() -> Result<Self, crate::error::CliError> {
        dotenv::dotenv().ok();

        let base_url =
            std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        let timeout_seconds: u64 = std::env::var("API_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .map_err(crate::error::CliError::HttpError)?;

        println!("🌐 API Client initialized: {}", base_url);

        Ok(Self { client, base_url })
    }

    pub async fn add_job(&self, job: JobRequestDTO) -> Result<ApiResponse, crate::error::CliError> {
        println!("🚀 Submitting job to: {}/jobs", self.base_url);

        let response = self
            .client
            .post(&format!("{}/jobs", self.base_url))
            .header("Content-Type", "application/json")
            .json(&job)
            .send()
            .await
            .map_err(crate::error::CliError::HttpError)?;

        if !response.status().is_success() {
            return Err(crate::error::CliError::HttpError(reqwest::Error::from(
                response.error_for_status().unwrap_err(),
            )));
        }

        response
            .json::<ApiResponse>()
            .await
            .map_err(crate::error::CliError::HttpError)
    }
}
