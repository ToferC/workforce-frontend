use graphql_client::{GraphQLQuery, Response};
use serde::{Serialize, Deserialize};
use std::error::Error;
use reqwest::Client;
use std::sync::Arc;

type UUID = String;

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/org_tiers/org_tier_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct OrgTierById;

pub async fn get_org_tier_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<org_tier_by_id::ResponseData, Box<dyn Error>> {

    let request_body = OrgTierById::build_query(org_tier_by_id::Variables {
        id,
    });

    let res = client
        .post(api_url)
        .header("Bearer", bearer)
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<org_tier_by_id::ResponseData> = res.json().await?;

    if let Some(errors) = response_body.errors {
        println!("there are errors:");

        for error in &errors {
            println!("{:?}", error);
        }
    };

    let response = response_body.data
        .expect("missing response data");

    Ok(response)
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/org_tiers/org_tiers_by_org_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct OrgTiersByOrgId;

pub async fn get_org_tiers_by_org_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<org_tiers_by_org_id::ResponseData, Box<dyn Error>> {

    let request_body = OrgTiersByOrgId::build_query(org_tiers_by_org_id::Variables {
        id,
    });

    let res = client
        .post(api_url)
        .header("Bearer", bearer)
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<org_tiers_by_org_id::ResponseData> = res.json().await?;

    if let Some(errors) = response_body.errors {
        println!("there are errors:");

        for error in &errors {
            println!("{:?}", error);
        }
    };

    let response = response_body.data
        .expect("missing response data");

    Ok(response)
}
