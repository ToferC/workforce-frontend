use graphql_client::GraphQLQuery;
use serde::{Serialize, Deserialize};
use reqwest::Client;
use std::sync::Arc;

use super::{post_graphql, ApiError};

type UUID = String;

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/organizations/organization_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct OrganizationById;

pub async fn get_organization_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<organization_by_id::ResponseData, ApiError> {
    post_graphql::<OrganizationById>(&client, api_url, &bearer, organization_by_id::Variables {
        id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/organizations/all_organizations.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct AllOrganizations;

pub async fn all_organizations(bearer: String, api_url: &str, client: Arc<Client>) -> Result<all_organizations::ResponseData, ApiError> {
    post_graphql::<AllOrganizations>(&client, api_url, &bearer, all_organizations::Variables {
    }).await
}
