use graphql_client::GraphQLQuery;
use serde::{Serialize, Deserialize};
use reqwest::Client;
use std::sync::Arc;

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
