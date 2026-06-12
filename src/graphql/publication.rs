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
    query_path = "queries/publications/publication_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct PublicationById;

pub async fn get_publication_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<publication_by_id::ResponseData, ApiError> {
    post_graphql::<PublicationById>(&client, api_url, &bearer, publication_by_id::Variables {
        id,
    }).await
}
