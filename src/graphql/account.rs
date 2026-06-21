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
