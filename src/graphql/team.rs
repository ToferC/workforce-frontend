use graphql_client::GraphQLQuery;
use serde::{Serialize, Deserialize};
use reqwest::Client;
use std::sync::Arc;

use super::{post_graphql, ApiError};

type UUID = String;

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/teams/team_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct TeamById;

pub async fn get_team_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<team_by_id::ResponseData, ApiError> {
    post_graphql::<TeamById>(&client, api_url, &bearer, team_by_id::Variables {
        id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/teams/all_teams.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct AllTeams;

pub async fn all_teams(bearer: String, api_url: &str, client: Arc<Client>) -> Result<all_teams::ResponseData, ApiError> {
    post_graphql::<AllTeams>(&client, api_url, &bearer, all_teams::Variables {
    }).await
}
