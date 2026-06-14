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
    query_path = "queries/people/person_by_name.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct PersonByName;

pub async fn get_people_by_name(name: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<person_by_name::ResponseData, ApiError> {
    post_graphql::<PersonByName>(&client, api_url, &bearer, person_by_name::Variables {
        name,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/people/person_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct PersonById;

pub async fn get_person_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<person_by_id::ResponseData, ApiError> {
    post_graphql::<PersonById>(&client, api_url, &bearer, person_by_id::Variables {
        id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/people/user_by_email.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct UserByEmail;

/// Look up the user account a new person record will be linked to.
pub async fn get_user_by_email(email: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<user_by_email::ResponseData, ApiError> {
    post_graphql::<UserByEmail>(&client, api_url, &bearer, user_by_email::Variables {
        email,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/people/create_person.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct CreatePerson;

pub async fn create_person(data: create_person::NewPerson, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_person::ResponseData, ApiError> {
    post_graphql::<CreatePerson>(&client, api_url, &bearer, create_person::Variables {
        data,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/people/update_person.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct UpdatePerson;

/// Also used to retire a person: pass retired_at = Some(now)
pub async fn update_person(data: update_person::PersonData, bearer: String, api_url: &str, client: Arc<Client>) -> Result<update_person::ResponseData, ApiError> {
    post_graphql::<UpdatePerson>(&client, api_url, &bearer, update_person::Variables {
        data,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/people/create_affiliation.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct CreateAffiliation;

pub async fn create_affiliation(data: create_affiliation::NewAffiliation, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_affiliation::ResponseData, ApiError> {
    post_graphql::<CreateAffiliation>(&client, api_url, &bearer, create_affiliation::Variables {
        data,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/people/update_affiliation.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct UpdateAffiliation;

/// End an affiliation by setting its end date.
pub async fn update_affiliation(data: update_affiliation::AffiliationData, bearer: String, api_url: &str, client: Arc<Client>) -> Result<update_affiliation::ResponseData, ApiError> {
    post_graphql::<UpdateAffiliation>(&client, api_url, &bearer, update_affiliation::Variables {
        data,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/people/all_people.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct AllPeople;

pub async fn all_people(bearer: String, api_url: &str, client: Arc<Client>) -> Result<all_people::ResponseData, ApiError> {
    post_graphql::<AllPeople>(&client, api_url, &bearer, all_people::Variables {}).await
}
