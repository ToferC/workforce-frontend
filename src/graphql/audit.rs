use graphql_client::GraphQLQuery;
use serde::{Serialize, Deserialize};
use reqwest::Client;
use std::sync::Arc;
use chrono::NaiveDateTime;

use super::{post_graphql, ApiError};

type UUID = String;

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/audit/recent_audit_events.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct RecentAuditEvents;

/// Most recent audit events across the system (admin only on the API).
pub async fn recent_audit_events(limit: i64, bearer: String, api_url: &str, client: Arc<Client>) -> Result<recent_audit_events::ResponseData, ApiError> {
    post_graphql::<RecentAuditEvents>(&client, api_url, &bearer, recent_audit_events::Variables { limit }).await
}
