use graphql_client::{GraphQLQuery, Response};
use serde::{Serialize, Deserialize};
use std::error::Error;
use reqwest::Client;
use std::sync::Arc;

type UUID = String;

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/organizations/organization_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct OrganizationById;

pub async fn get_organization_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<organization_by_id::ResponseData, Box<dyn Error>> {

    let request_body = OrganizationById::build_query(organization_by_id::Variables {
        id,
    });

    let res = client
        .post(api_url)
        .header("Bearer", bearer)
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<organization_by_id::ResponseData> = res.json().await?;

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
    query_path = "queries/organizations/all_organizations.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct AllOrganizations;

pub async fn all_organizations(bearer: String, api_url: &str, client: Arc<Client>) -> Result<all_organizations::ResponseData, Box<dyn Error>> {

    let request_body = AllOrganizations::build_query(all_organizations::Variables {
    });

    let res = client
        .post(api_url)
        .header("Bearer", bearer)
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<all_organizations::ResponseData> = res.json().await?;

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
