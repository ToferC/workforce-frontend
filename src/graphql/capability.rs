use graphql_client::GraphQLQuery;
use serde::{Serialize, Deserialize};
use reqwest::Client;
use std::sync::Arc;
use chrono::NaiveDateTime;

use super::{post_graphql, ApiError};

type UUID = String;

#[derive(GraphQLQuery, Serialize, Deserialize, Clone, Copy)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/capabilities/capability_by_name_and_level.graphql",
    response_derives = "Debug, Serialize, PartialEq, Deserialize"
)]
pub struct CapabilityByNameAndLevel;

pub async fn get_capability_by_name_and_level(name: String, level: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<capability_by_name_and_level::ResponseData, ApiError> {
    post_graphql::<CapabilityByNameAndLevel>(&client, api_url, &bearer, capability_by_name_and_level::Variables {
        name,
        level,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/capabilities/create_capability.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct CreateCapability;

pub async fn create_capability(data: create_capability::NewCapability, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_capability::ResponseData, ApiError> {
    post_graphql::<CreateCapability>(&client, api_url, &bearer, create_capability::Variables { data }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/capabilities/update_capability.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct UpdateCapability;

/// Update a capability's self-identified level, or retire it
/// (retired_at = now).
pub async fn update_capability(data: update_capability::CapabilityData, bearer: String, api_url: &str, client: Arc<Client>) -> Result<update_capability::ResponseData, ApiError> {
    post_graphql::<UpdateCapability>(&client, api_url, &bearer, update_capability::Variables { data }).await
}
