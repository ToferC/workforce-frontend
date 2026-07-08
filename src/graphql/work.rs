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

/// `status` is the WorkStatus enum value as a string (e.g. "IN_PROGRESS"), or
/// None for no status filter — an unrecognized value is treated as no filter.
/// `limit` = None returns every matching row (used by the vacancy dashboard).
pub async fn all_work(
    status: Option<String>,
    unassigned_only: bool,
    limit: Option<i64>,
    offset: i64,
    bearer: String,
    api_url: &str,
    client: Arc<Client>,
) -> Result<all_work::ResponseData, ApiError> {
    let status = status.and_then(|s| serde_json::from_value::<all_work::WorkStatus>(serde_json::json!(s)).ok());
    post_graphql::<AllWork>(&client, api_url, &bearer, all_work::Variables {
        status,
        unassigned_only,
        limit,
        offset,
    }).await
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

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/work/my_work.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct MyWork;

pub async fn my_work(bearer: String, api_url: &str, client: Arc<Client>) -> Result<my_work::ResponseData, ApiError> {
    post_graphql::<MyWork>(&client, api_url, &bearer, my_work::Variables {}).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/work/add_work_update.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct AddWorkUpdate;

pub async fn add_work_update(work_id: String, body: String, kind: Option<add_work_update::WorkUpdateKind>, bearer: String, api_url: &str, client: Arc<Client>) -> Result<add_work_update::ResponseData, ApiError> {
    post_graphql::<AddWorkUpdate>(&client, api_url, &bearer, add_work_update::Variables { work_id, body, kind }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/work/resolve_work_update_flag.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct ResolveWorkUpdateFlag;

pub async fn resolve_work_update_flag(update_id: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<resolve_work_update_flag::ResponseData, ApiError> {
    post_graphql::<ResolveWorkUpdateFlag>(&client, api_url, &bearer, resolve_work_update_flag::Variables { update_id }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/work/open_work_flags.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct OpenWorkFlags;

pub async fn open_work_flags(limit: Option<i64>, bearer: String, api_url: &str, client: Arc<Client>) -> Result<open_work_flags::ResponseData, ApiError> {
    post_graphql::<OpenWorkFlags>(&client, api_url, &bearer, open_work_flags::Variables { limit }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/work/add_work_dependency.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct AddWorkDependency;

pub async fn add_work_dependency(work_id: String, depends_on_work_id: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<add_work_dependency::ResponseData, ApiError> {
    post_graphql::<AddWorkDependency>(&client, api_url, &bearer, add_work_dependency::Variables { work_id, depends_on_work_id }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/work/remove_work_dependency.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct RemoveWorkDependency;

pub async fn remove_work_dependency(work_id: String, depends_on_work_id: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<remove_work_dependency::ResponseData, ApiError> {
    post_graphql::<RemoveWorkDependency>(&client, api_url, &bearer, remove_work_dependency::Variables { work_id, depends_on_work_id }).await
}
