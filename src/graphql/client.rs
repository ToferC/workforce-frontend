use graphql_client::GraphQLQuery;
use reqwest::Client;
use std::fmt;

/// Errors returned when calling the workforce_analytics GraphQL API.
#[derive(Debug)]
pub enum ApiError {
    /// Network or deserialization failure from reqwest
    Request(reqwest::Error),
    /// The API responded, but with GraphQL errors (e.g. permission denied)
    GraphQL(Vec<graphql_client::Error>),
    /// The API responded without errors but also without data
    MissingData,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::Request(e) => write!(f, "API request failed: {}", e),
            ApiError::GraphQL(errors) => {
                let messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
                write!(f, "{}", messages.join("; "))
            }
            ApiError::MissingData => write!(f, "API response contained no data"),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        ApiError::Request(e)
    }
}

/// Send a GraphQL query or mutation to the API and return its data.
///
/// All API calls should go through this function: it attaches the JWT as
/// `Authorization: Bearer <token>` (the header the API actually validates)
/// and surfaces GraphQL errors instead of panicking on missing data.
pub async fn post_graphql<Q: GraphQLQuery>(
    client: &Client,
    api_url: &str,
    bearer: &str,
    variables: Q::Variables,
) -> Result<Q::ResponseData, ApiError> {
    let request_body = Q::build_query(variables);

    let mut request = client.post(api_url).json(&request_body);

    if !bearer.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", bearer));
    }

    let response_body: graphql_client::Response<Q::ResponseData> =
        request.send().await?.json().await?;

    if let Some(errors) = response_body.errors {
        if !errors.is_empty() {
            println!("GraphQL errors: {:?}", &errors);
            return Err(ApiError::GraphQL(errors));
        }
    }

    response_body.data.ok_or(ApiError::MissingData)
}
