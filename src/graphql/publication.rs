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

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/publications/all_publications.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct AllPublications;

pub async fn all_publications(bearer: String, api_url: &str, client: Arc<Client>) -> Result<all_publications::ResponseData, ApiError> {
    post_graphql::<AllPublications>(&client, api_url, &bearer, all_publications::Variables {}).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/publications/create_publication.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct CreatePublication;

pub async fn create_publication(data: create_publication::NewPublication, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_publication::ResponseData, ApiError> {
    post_graphql::<CreatePublication>(&client, api_url, &bearer, create_publication::Variables { data }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/publications/update_publication.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct UpdatePublication;

pub async fn update_publication(data: update_publication::PublicationData, bearer: String, api_url: &str, client: Arc<Client>) -> Result<update_publication::ResponseData, ApiError> {
    post_graphql::<UpdatePublication>(&client, api_url, &bearer, update_publication::Variables { data }).await
}
