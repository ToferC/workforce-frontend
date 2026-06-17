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
    query_path = "queries/roles/role_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct RoleById;

pub async fn get_role_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<role_by_id::ResponseData, ApiError> {
    post_graphql::<RoleById>(&client, api_url, &bearer, role_by_id::Variables {
        id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/roles/role_fuzzy_matches.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct RoleFuzzyMatches;

/// Tiered, scored candidate matches for a role. `min_coverage` is the minimum
/// fraction of requirements a partial candidate must meet (0.0–1.0);
/// `max_gap_per_req` is the largest single-skill shortfall tolerated before a
/// candidate is dropped entirely. Drives the vacant-role matching UI.
pub async fn get_role_matches(
    id: UUID,
    min_coverage: f64,
    max_gap_per_req: i64,
    limit: i64,
    bearer: String,
    api_url: &str,
    client: Arc<Client>,
) -> Result<role_fuzzy_matches::ResponseData, ApiError> {
    post_graphql::<RoleFuzzyMatches>(&client, api_url, &bearer, role_fuzzy_matches::Variables {
        id,
        min_coverage,
        max_gap_per_req,
        limit,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/roles/create_role.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct CreateRole;

pub async fn create_role(data: create_role::NewRole, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_role::ResponseData, ApiError> {
    post_graphql::<CreateRole>(&client, api_url, &bearer, create_role::Variables {
        data,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/roles/update_role.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct UpdateRole;

/// The API only allows updating active/startDatestamp/endDate by design
/// (create a new role instead of rewriting history). Ending a role is
/// active = false + endDate = now.
pub async fn update_role(data: update_role::RoleData, bearer: String, api_url: &str, client: Arc<Client>) -> Result<update_role::ResponseData, ApiError> {
    post_graphql::<UpdateRole>(&client, api_url, &bearer, update_role::Variables {
        data,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/roles/assign_person_to_role.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct AssignPersonToRole;

/// Assign a person to a vacant role. The API errors if the role is already
/// occupied, surfaced here as an ApiError::GraphQL.
pub async fn assign_person_to_role(person_id: UUID, role_id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<assign_person_to_role::ResponseData, ApiError> {
    post_graphql::<AssignPersonToRole>(&client, api_url, &bearer, assign_person_to_role::Variables {
        person_id,
        role_id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/roles/vacate_role.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct VacateRole;

/// Remove the person from a role, leaving it vacant.
pub async fn vacate_role(role_id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<vacate_role::ResponseData, ApiError> {
    post_graphql::<VacateRole>(&client, api_url, &bearer, vacate_role::Variables {
        role_id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/roles/all_roles.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct AllRoles;

pub async fn all_roles(bearer: String, api_url: &str, client: Arc<Client>) -> Result<all_roles::ResponseData, ApiError> {
    post_graphql::<AllRoles>(&client, api_url, &bearer, all_roles::Variables {}).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/roles/vacant_roles.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct VacantRoles;

pub async fn vacant_roles(count: i64, bearer: String, api_url: &str, client: Arc<Client>) -> Result<vacant_roles::ResponseData, ApiError> {
    post_graphql::<VacantRoles>(&client, api_url, &bearer, vacant_roles::Variables { count }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/analytics/analytics_roles.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct AnalyticsRoles;

pub async fn analytics_roles(bearer: String, api_url: &str, client: Arc<Client>) -> Result<analytics_roles::ResponseData, ApiError> {
    post_graphql::<AnalyticsRoles>(&client, api_url, &bearer, analytics_roles::Variables {}).await
}
