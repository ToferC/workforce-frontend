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
    query_path = "queries/roles/role_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct RoleById;

pub async fn get_role_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<role_by_id::ResponseData, ApiError> {
    post_graphql::<RoleById>(&client, api_url, &bearer, role_by_id::Variables {
        id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/roles/create_role.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct CreateRole;

pub async fn create_role(data: create_role::NewRole, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_role::ResponseData, ApiError> {
    post_graphql::<CreateRole>(&client, api_url, &bearer, create_role::Variables {
        data,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/roles/update_role.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct UpdateRole;

/// The API only allows updating active/startDatestamp/endDate by design
/// (create a new role instead of rewriting history). Ending a role is
/// active = false + endDate = now.
pub async fn update_role(data: update_role::RoleData, bearer: String, api_url: &str, client: Arc<Client>) -> Result<update_role::ResponseData, ApiError> {
    post_graphql::<UpdateRole>(&client, api_url, &bearer, update_role::Variables {
        data,
    }).await
}
