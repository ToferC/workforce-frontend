use graphql_client::GraphQLQuery;
use serde::{Serialize, Deserialize};
use reqwest::Client;
use std::sync::Arc;
use chrono::NaiveDateTime;

use crate::graphql::log_in_mutation;
use crate::graphql::log_in::LoginQuery;
use super::{post_graphql, ApiError};

type UUID = String;

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/log_in.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct LogIn;

pub async fn login(email: String, password: String, api_url: &str, client: Arc<Client>) -> Result<log_in::ResponseData, ApiError> {

    let auth_data = log_in_mutation::LoginQuery {
        email,
        password,
    };

    // No bearer token yet — signing in is what issues it
    post_graphql::<LogIn>(&client, api_url, "", log_in::Variables {
        auth_data,
    }).await
}
