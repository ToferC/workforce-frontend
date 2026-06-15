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
    query_path = "queries/organizations/organization_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct OrganizationById;

pub async fn get_organization_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<organization_by_id::ResponseData, ApiError> {
    post_graphql::<OrganizationById>(&client, api_url, &bearer, organization_by_id::Variables {
        id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/organizations/all_organizations.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct AllOrganizations;

pub async fn all_organizations(bearer: String, api_url: &str, client: Arc<Client>) -> Result<all_organizations::ResponseData, ApiError> {
    post_graphql::<AllOrganizations>(&client, api_url, &bearer, all_organizations::Variables {
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/organizations/create_organization.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct CreateOrganization;

pub async fn create_organization(data: create_organization::NewOrganization, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_organization::ResponseData, ApiError> {
    post_graphql::<CreateOrganization>(&client, api_url, &bearer, create_organization::Variables {
        data,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/organizations/update_organization.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct UpdateOrganization;

/// Also used to retire an organization: pass retired_at = Some(now)
/// (the API has no delete mutations, deletion is a soft retire).
pub async fn update_organization(data: update_organization::OrganizationData, bearer: String, api_url: &str, client: Arc<Client>) -> Result<update_organization::ResponseData, ApiError> {
    post_graphql::<UpdateOrganization>(&client, api_url, &bearer, update_organization::Variables {
        data,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/organizations/restore_organization.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct RestoreOrganization;

pub async fn restore_organization(id: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<restore_organization::ResponseData, ApiError> {
    post_graphql::<RestoreOrganization>(&client, api_url, &bearer, restore_organization::Variables { id }).await
}
