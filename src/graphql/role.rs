use graphql_client::GraphQLQuery;
use serde::{Serialize, Deserialize};
use reqwest::Client;
use std::sync::Arc;

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
