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
    query_path = "queries/finance/create_contract.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct CreateContract;

pub async fn create_contract(data: create_contract::NewContract, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_contract::ResponseData, ApiError> {
    post_graphql::<CreateContract>(&client, api_url, &bearer, create_contract::Variables { data }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/finance/update_contract.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct UpdateContract;

pub async fn update_contract(data: update_contract::ContractUpdate, bearer: String, api_url: &str, client: Arc<Client>) -> Result<update_contract::ResponseData, ApiError> {
    post_graphql::<UpdateContract>(&client, api_url, &bearer, update_contract::Variables { data }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/finance/delete_contract.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct DeleteContract;

pub async fn delete_contract(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<delete_contract::ResponseData, ApiError> {
    post_graphql::<DeleteContract>(&client, api_url, &bearer, delete_contract::Variables { id }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/finance/contract_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct ContractById;

pub async fn get_contract_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<contract_by_id::ResponseData, ApiError> {
    post_graphql::<ContractById>(&client, api_url, &bearer, contract_by_id::Variables { id }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/finance/pay_rates.graphql",
    response_derives = "Debug, Serialize, PartialEq, Clone"
)]
pub struct PayRates;

pub async fn all_pay_rates(bearer: String, api_url: &str, client: Arc<Client>) -> Result<pay_rates::ResponseData, ApiError> {
    post_graphql::<PayRates>(&client, api_url, &bearer, pay_rates::Variables {}).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/finance/create_pay_rate.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct CreatePayRate;

pub async fn create_pay_rate(data: create_pay_rate::NewPayRate, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_pay_rate::ResponseData, ApiError> {
    post_graphql::<CreatePayRate>(&client, api_url, &bearer, create_pay_rate::Variables { data }).await
}
