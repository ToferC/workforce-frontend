use graphql_client::GraphQLQuery;
use serde::{Serialize, Deserialize};
use reqwest::Client;
use std::sync::Arc;

use super::{post_graphql, ApiError};

type UUID = String;

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/languages/create_language_data.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct CreateLanguageData;

pub async fn create_language_data(data: create_language_data::NewLanguageData, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_language_data::ResponseData, ApiError> {
    post_graphql::<CreateLanguageData>(&client, api_url, &bearer, create_language_data::Variables { data }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/languages/update_language_data.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct UpdateLanguageData;

pub async fn update_language_data(data: update_language_data::LanguageDataUpdate, bearer: String, api_url: &str, client: Arc<Client>) -> Result<update_language_data::ResponseData, ApiError> {
    post_graphql::<UpdateLanguageData>(&client, api_url, &bearer, update_language_data::Variables { data }).await
}
