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
    query_path = "queries/requirements/create_requirement.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct CreateRequirement;

pub async fn create_requirement(data: create_requirement::NewRequirement, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_requirement::ResponseData, ApiError> {
    post_graphql::<CreateRequirement>(&client, api_url, &bearer, create_requirement::Variables { data }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/requirements/update_requirement.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct UpdateRequirement;

/// Also used to retire a requirement: pass retired_at = Some(now).
pub async fn update_requirement(data: update_requirement::RequirementData, bearer: String, api_url: &str, client: Arc<Client>) -> Result<update_requirement::ResponseData, ApiError> {
    post_graphql::<UpdateRequirement>(&client, api_url, &bearer, update_requirement::Variables { data }).await
}
