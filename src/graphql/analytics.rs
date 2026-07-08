use graphql_client::GraphQLQuery;
use serde::{Serialize, Deserialize};
use reqwest::Client;
use std::sync::Arc;
use chrono::NaiveDateTime;

use super::{post_graphql, ApiError};

type UUID = String;

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/analytics/team_capability_matrix.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct TeamCapabilityMatrix;

pub async fn team_capability_matrix(org_tier_id: Option<String>, bearer: String, api_url: &str, client: Arc<Client>) -> Result<team_capability_matrix::ResponseData, ApiError> {
    post_graphql::<TeamCapabilityMatrix>(&client, api_url, &bearer, team_capability_matrix::Variables {
        org_tier_id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/analytics/talent_movements.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct TalentMovements;

pub async fn talent_movements(from: Option<NaiveDateTime>, to: Option<NaiveDateTime>, org_tier_id: Option<String>, bearer: String, api_url: &str, client: Arc<Client>) -> Result<talent_movements::ResponseData, ApiError> {
    post_graphql::<TalentMovements>(&client, api_url, &bearer, talent_movements::Variables {
        from,
        to,
        org_tier_id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/analytics/capability_growth.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct CapabilityGrowth;

pub async fn capability_growth(bucket: capability_growth::TimeBucket, from: Option<NaiveDateTime>, to: Option<NaiveDateTime>, org_tier_id: Option<String>, bearer: String, api_url: &str, client: Arc<Client>) -> Result<capability_growth::ResponseData, ApiError> {
    post_graphql::<CapabilityGrowth>(&client, api_url, &bearer, capability_growth::Variables {
        bucket,
        from,
        to,
        org_tier_id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/analytics/capability_supply_demand.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct CapabilitySupplyDemand;

pub async fn capability_supply_demand(bucket: capability_supply_demand::TimeBucket, from: Option<NaiveDateTime>, to: Option<NaiveDateTime>, org_tier_id: Option<String>, bearer: String, api_url: &str, client: Arc<Client>) -> Result<capability_supply_demand::ResponseData, ApiError> {
    post_graphql::<CapabilitySupplyDemand>(&client, api_url, &bearer, capability_supply_demand::Variables {
        bucket,
        from,
        to,
        org_tier_id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/analytics/priority_mismatches.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct PriorityMismatches;

pub async fn priority_mismatches(bearer: String, api_url: &str, client: Arc<Client>) -> Result<priority_mismatches::ResponseData, ApiError> {
    post_graphql::<PriorityMismatches>(&client, api_url, &bearer, priority_mismatches::Variables {}).await
}
