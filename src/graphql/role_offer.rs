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
    query_path = "queries/role_offers/incoming_role_offers.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct IncomingRoleOffers;

/// Pending transfer offers awaiting the caller's decision (the API scopes these
/// to the candidates the caller manages).
pub async fn incoming_role_offers(bearer: String, api_url: &str, client: Arc<Client>) -> Result<incoming_role_offers::ResponseData, ApiError> {
    post_graphql::<IncomingRoleOffers>(&client, api_url, &bearer, incoming_role_offers::Variables {}).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/role_offers/outgoing_role_offers.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct OutgoingRoleOffers;

/// Transfer offers the caller has made (any status).
pub async fn outgoing_role_offers(bearer: String, api_url: &str, client: Arc<Client>) -> Result<outgoing_role_offers::ResponseData, ApiError> {
    post_graphql::<OutgoingRoleOffers>(&client, api_url, &bearer, outgoing_role_offers::Variables {}).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/role_offers/create_role_offer.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct CreateRoleOffer;

/// Offer a vacant role to a candidate outside the caller's managed area.
pub async fn create_role_offer(role_id: UUID, person_id: UUID, message: Option<String>, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_role_offer::ResponseData, ApiError> {
    post_graphql::<CreateRoleOffer>(&client, api_url, &bearer, create_role_offer::Variables {
        role_id,
        person_id,
        message,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/role_offers/accept_role_offer.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct AcceptRoleOffer;

/// Accept a pending offer (executes the transfer). Caller must manage the
/// candidate's current team.
pub async fn accept_role_offer(offer_id: UUID, note: Option<String>, bearer: String, api_url: &str, client: Arc<Client>) -> Result<accept_role_offer::ResponseData, ApiError> {
    post_graphql::<AcceptRoleOffer>(&client, api_url, &bearer, accept_role_offer::Variables {
        offer_id,
        note,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/role_offers/decline_role_offer.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct DeclineRoleOffer;

/// Decline a pending offer. Caller must manage the candidate's current team.
pub async fn decline_role_offer(offer_id: UUID, note: Option<String>, bearer: String, api_url: &str, client: Arc<Client>) -> Result<decline_role_offer::ResponseData, ApiError> {
    post_graphql::<DeclineRoleOffer>(&client, api_url, &bearer, decline_role_offer::Variables {
        offer_id,
        note,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/role_offers/withdraw_role_offer.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct WithdrawRoleOffer;

/// Withdraw a pending offer. Caller must manage the hiring team.
pub async fn withdraw_role_offer(offer_id: UUID, note: Option<String>, bearer: String, api_url: &str, client: Arc<Client>) -> Result<withdraw_role_offer::ResponseData, ApiError> {
    post_graphql::<WithdrawRoleOffer>(&client, api_url, &bearer, withdraw_role_offer::Variables {
        offer_id,
        note,
    }).await
}
