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
    query_path = "queries/org_tiers/org_tier_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct OrgTierById;

pub async fn get_org_tier_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<org_tier_by_id::ResponseData, ApiError> {
    post_graphql::<OrgTierById>(&client, api_url, &bearer, org_tier_by_id::Variables {
        id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/org_tiers/org_tiers_by_org_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct OrgTiersByOrgId;

pub async fn get_org_tiers_by_org_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<org_tiers_by_org_id::ResponseData, ApiError> {
    post_graphql::<OrgTiersByOrgId>(&client, api_url, &bearer, org_tiers_by_org_id::Variables {
        id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/org_tiers/org_tier_node.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct OrgTierNode;

/// Fetch one expandable org chart node: child tiers plus teams with
/// occupied and vacant roles.
pub async fn get_org_tier_node(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<org_tier_node::ResponseData, ApiError> {
    post_graphql::<OrgTierNode>(&client, api_url, &bearer, org_tier_node::Variables {
        id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/org_tiers/create_org_tier.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct CreateOrgTier;

pub async fn create_org_tier(data: create_org_tier::NewOrgTier, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_org_tier::ResponseData, ApiError> {
    post_graphql::<CreateOrgTier>(&client, api_url, &bearer, create_org_tier::Variables {
        data,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/org_tiers/update_org_tier.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct UpdateOrgTier;

/// Also used to retire an org tier: pass retired_at = Some(now)
pub async fn update_org_tier(data: update_org_tier::OrgTierData, bearer: String, api_url: &str, client: Arc<Client>) -> Result<update_org_tier::ResponseData, ApiError> {
    post_graphql::<UpdateOrgTier>(&client, api_url, &bearer, update_org_tier::Variables {
        data,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/org_tiers/create_org_ownership.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct CreateOrgOwnership;

/// Assign an owner to an org tier. Tiers created via createOrgTier have no
/// ownership record (owner() then inherits up the parent chain); this
/// creates one so the tier has its own owner.
pub async fn create_org_ownership(data: create_org_ownership::NewOrgOwnership, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_org_ownership::ResponseData, ApiError> {
    post_graphql::<CreateOrgOwnership>(&client, api_url, &bearer, create_org_ownership::Variables {
        data,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/org_tiers/org_ownership_by_tier_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct OrgOwnershipByTierId;

/// Look up a tier's ownership record (and its id) so the owner can be
/// reassigned. Returns an ApiError if the tier has no ownership record.
pub async fn get_org_ownership_by_tier_id(org_tier_id: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<org_ownership_by_tier_id::ResponseData, ApiError> {
    post_graphql::<OrgOwnershipByTierId>(&client, api_url, &bearer, org_ownership_by_tier_id::Variables {
        org_tier_id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/org_tiers/update_org_ownership.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct UpdateOrgOwnership;

pub async fn update_org_ownership(data: update_org_ownership::OrgOwnershipData, bearer: String, api_url: &str, client: Arc<Client>) -> Result<update_org_ownership::ResponseData, ApiError> {
    post_graphql::<UpdateOrgOwnership>(&client, api_url, &bearer, update_org_ownership::Variables {
        data,
    }).await
}
