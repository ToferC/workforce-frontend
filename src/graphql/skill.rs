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
    query_path = "queries/skills/all_skills.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct AllSkills;

pub async fn all_skills(bearer: String, api_url: &str, client: Arc<Client>) -> Result<all_skills::ResponseData, ApiError> {
    post_graphql::<AllSkills>(&client, api_url, &bearer, all_skills::Variables {}).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/skills/skill_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct SkillById;

pub async fn get_skill_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<skill_by_id::ResponseData, ApiError> {
    post_graphql::<SkillById>(&client, api_url, &bearer, skill_by_id::Variables { id }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/skills/create_skill.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct CreateSkill;

pub async fn create_skill(data: create_skill::NewSkill, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_skill::ResponseData, ApiError> {
    post_graphql::<CreateSkill>(&client, api_url, &bearer, create_skill::Variables { data }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/skills/update_skill.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct UpdateSkill;

pub async fn update_skill(data: update_skill::SkillData, bearer: String, api_url: &str, client: Arc<Client>) -> Result<update_skill::ResponseData, ApiError> {
    post_graphql::<UpdateSkill>(&client, api_url, &bearer, update_skill::Variables { data }).await
}
