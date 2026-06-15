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
    query_path = "queries/work/work_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct WorkById;

pub async fn get_work_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<work_by_id::ResponseData, ApiError> {
    post_graphql::<WorkById>(&client, api_url, &bearer, work_by_id::Variables {
        id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/work/all_work.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct AllWork;

pub async fn all_work(bearer: String, api_url: &str, client: Arc<Client>) -> Result<all_work::ResponseData, ApiError> {
    post_graphql::<AllWork>(&client, api_url, &bearer, all_work::Variables {}).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/work/create_work.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct CreateWork;

pub async fn create_work(data: create_work::NewWork, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_work::ResponseData, ApiError> {
    post_graphql::<CreateWork>(&client, api_url, &bearer, create_work::Variables { data }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/work/update_work.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct UpdateWork;

pub async fn update_work(data: update_work::WorkData, bearer: String, api_url: &str, client: Arc<Client>) -> Result<update_work::ResponseData, ApiError> {
    post_graphql::<UpdateWork>(&client, api_url, &bearer, update_work::Variables { data }).await
}
