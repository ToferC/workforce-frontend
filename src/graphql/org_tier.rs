use graphql_client::GraphQLQuery;
use serde::{Serialize, Deserialize};
use reqwest::Client;
use std::sync::Arc;

use super::{post_graphql, ApiError};

type UUID = String;

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/org_tiers/org_tier_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct OrgTierById;

pub async fn get_org_tier_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<org_tier_by_id::ResponseData, ApiError> {
    post_graphql::<OrgTierById>(&client, api_url, &bearer, org_tier_by_id::Variables {
        id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/org_tiers/org_tiers_by_org_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct OrgTiersByOrgId;

pub async fn get_org_tiers_by_org_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<org_tiers_by_org_id::ResponseData, ApiError> {
    post_graphql::<OrgTiersByOrgId>(&client, api_url, &bearer, org_tiers_by_org_id::Variables {
        id,
    }).await
}
