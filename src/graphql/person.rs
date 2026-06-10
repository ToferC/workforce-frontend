use graphql_client::{GraphQLQuery, Response};
use serde::{Serialize, Deserialize};
use std::error::Error;
use reqwest::Client;
use std::sync::Arc;
use chrono::NaiveDateTime;

type UUID = String;

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/people/person_by_name.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct PersonByName;

pub async fn get_people_by_name(name: String, bearer: String, api_url: &str, client: Arc<Client>) -> Result<person_by_name::ResponseData, Box<dyn Error>> {

    let request_body = PersonByName::build_query(person_by_name::Variables {
        name,
    });

    let res = client
        .post(api_url)
        .header("Bearer", bearer)
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<person_by_name::ResponseData> = res.json().await?;

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
    query_path = "queries/people/person_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct PersonById;

pub async fn get_person_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<person_by_id::ResponseData, Box<dyn Error>> {

    let request_body = PersonById::build_query(person_by_id::Variables {
        id,
    });

    let res = client
        .post(api_url)
        .header("Bearer", bearer)
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<person_by_id::ResponseData> = res.json().await?;

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
