use graphql_client::GraphQLQuery;
use serde::{Serialize, Deserialize};
use reqwest::Client;
use std::sync::Arc;

use super::{post_graphql, ApiError};

type UUID = String;

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/validations/create_validation.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct CreateValidation;

/// Admin-only on the API. Creating a validation makes the API recalculate
/// the capability's validated level as the average of its validations.
pub async fn create_validation(data: create_validation::NewValidation, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_validation::ResponseData, ApiError> {
    post_graphql::<CreateValidation>(&client, api_url, &bearer, create_validation::Variables { data }).await
}
