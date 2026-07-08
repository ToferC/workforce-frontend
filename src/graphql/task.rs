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
    query_path = "queries/task/task_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct TaskById;

pub async fn get_task_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<task_by_id::ResponseData, ApiError> {
    post_graphql::<TaskById>(&client, api_url, &bearer, task_by_id::Variables {
        id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/task/all_tasks.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct AllTasks;

pub async fn all_tasks(bearer: String, api_url: &str, client: Arc<Client>) -> Result<all_tasks::ResponseData, ApiError> {
    post_graphql::<AllTasks>(&client, api_url, &bearer, all_tasks::Variables {}).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/task/create_task.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct CreateTask;

pub async fn create_task(data: create_task::NewTask, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_task::ResponseData, ApiError> {
    post_graphql::<CreateTask>(&client, api_url, &bearer, create_task::Variables { data }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/task/update_task.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct UpdateTask;

pub async fn update_task(data: update_task::TaskData, bearer: String, api_url: &str, client: Arc<Client>) -> Result<update_task::ResponseData, ApiError> {
    post_graphql::<UpdateTask>(&client, api_url, &bearer, update_task::Variables { data }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/task/submit_task_for_approval.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct SubmitTaskForApproval;

pub async fn submit_task_for_approval(task_id: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<submit_task_for_approval::ResponseData, ApiError> {
    post_graphql::<SubmitTaskForApproval>(&client, api_url, &bearer, submit_task_for_approval::Variables { task_id }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/task/approve_task.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct ApproveTask;

pub async fn approve_task(task_id: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<approve_task::ResponseData, ApiError> {
    post_graphql::<ApproveTask>(&client, api_url, &bearer, approve_task::Variables { task_id }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/task/reject_task.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct RejectTask;

pub async fn reject_task(task_id: String, reason: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<reject_task::ResponseData, ApiError> {
    post_graphql::<RejectTask>(&client, api_url, &bearer, reject_task::Variables { task_id, reason }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/task/pending_approvals.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct PendingApprovals;

pub async fn pending_approvals(limit: Option<i64>, bearer: String, api_url: &str, client: Arc<Client>) -> Result<pending_approvals::ResponseData, ApiError> {
    post_graphql::<PendingApprovals>(&client, api_url, &bearer, pending_approvals::Variables { limit }).await
}
