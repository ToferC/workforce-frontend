use graphql_client::GraphQLQuery;
use serde::{Serialize, Deserialize};
use reqwest::Client;
use std::sync::Arc;
use chrono::NaiveDateTime;

use super::{post_graphql, ApiError};

type UUID = String;

/// Issue an activation invite for a provisioned user (operator/admin). Returns
/// the activation token so the caller can build an activation link.
#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/users/invite_user.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct InviteUser;

pub async fn invite_user(user_id: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<invite_user::ResponseData, ApiError> {
    post_graphql::<InviteUser>(&client, api_url, &bearer, invite_user::Variables { user_id }).await
}

/// Redeem an activation token by setting a password (public — no bearer).
#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/users/activate_account.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct ActivateAccount;

pub async fn activate_account(token: String, password: String, api_url: &str, client: Arc<Client>) -> Result<activate_account::ResponseData, ApiError> {
    post_graphql::<ActivateAccount>(&client, api_url, "", activate_account::Variables { token, password }).await
}

/// The authenticated caller's account + linked person (self-service).
#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/me.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct Me;

pub async fn get_me(bearer: String, api_url: &str, client: Arc<Client>) -> Result<me::ResponseData, ApiError> {
    post_graphql::<Me>(&client, api_url, &bearer, me::Variables {}).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/people/update_my_person.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct UpdateMyPerson;

pub async fn update_my_person(data: update_my_person::MyPersonUpdate, bearer: String, api_url: &str, client: Arc<Client>) -> Result<update_my_person::ResponseData, ApiError> {
    post_graphql::<UpdateMyPerson>(&client, api_url, &bearer, update_my_person::Variables { data }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/people/flag_record_issue.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct FlagRecordIssue;

pub async fn flag_record_issue(message: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<flag_record_issue::ResponseData, ApiError> {
    post_graphql::<FlagRecordIssue>(&client, api_url, &bearer, flag_record_issue::Variables { message }).await
}

// `NaiveDateTime` is referenced by the InviteUser response (expiresAt); keep the
// import used so the scalar resolves to chrono's type.
#[allow(dead_code)]
type ActivationExpiry = NaiveDateTime;

// ── Admin user management ────────────────────────────────────────────────────

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/users/all_users.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct AllUsers;

pub async fn all_users(bearer: String, api_url: &str, client: Arc<Client>) -> Result<all_users::ResponseData, ApiError> {
    post_graphql::<AllUsers>(&client, api_url, &bearer, all_users::Variables {}).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/users/user_by_id.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct UserById;

pub async fn get_user_by_id(id: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<user_by_id::ResponseData, ApiError> {
    post_graphql::<UserById>(&client, api_url, &bearer, user_by_id::Variables { id }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/users/create_user.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct CreateUser;

pub async fn create_user(user_data: create_user::UserData, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_user::ResponseData, ApiError> {
    post_graphql::<CreateUser>(&client, api_url, &bearer, create_user::Variables { user_data }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/users/update_user.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct UpdateUser;

pub async fn update_user(user_data: update_user::UserUpdate, bearer: String, api_url: &str, client: Arc<Client>) -> Result<update_user::ResponseData, ApiError> {
    post_graphql::<UpdateUser>(&client, api_url, &bearer, update_user::Variables { user_data }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/users/disable_user.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct DisableUser;

pub async fn disable_user(user_id: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<disable_user::ResponseData, ApiError> {
    post_graphql::<DisableUser>(&client, api_url, &bearer, disable_user::Variables { user_id }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/users/enable_user.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct EnableUser;

pub async fn enable_user(user_id: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<enable_user::ResponseData, ApiError> {
    post_graphql::<EnableUser>(&client, api_url, &bearer, enable_user::Variables { user_id }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/users/record_flags.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct RecordFlags;

pub async fn record_flags(bearer: String, api_url: &str, client: Arc<Client>) -> Result<record_flags::ResponseData, ApiError> {
    post_graphql::<RecordFlags>(&client, api_url, &bearer, record_flags::Variables {}).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/users/resolve_record_flag.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct ResolveRecordFlag;

pub async fn resolve_record_flag(id: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<resolve_record_flag::ResponseData, ApiError> {
    post_graphql::<ResolveRecordFlag>(&client, api_url, &bearer, resolve_record_flag::Variables { id }).await
}

