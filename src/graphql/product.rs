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
    query_path = "queries/products/product_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct ProductById;

pub async fn get_product_by_id(id: UUID, bearer: String, api_url: &str, client: Arc<Client>) -> Result<product_by_id::ResponseData, ApiError> {
    post_graphql::<ProductById>(&client, api_url, &bearer, product_by_id::Variables {
        id,
    }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/products/all_products.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct AllProducts;

pub async fn all_products(bearer: String, api_url: &str, client: Arc<Client>) -> Result<all_products::ResponseData, ApiError> {
    post_graphql::<AllProducts>(&client, api_url, &bearer, all_products::Variables {}).await
}

/// Lean list for owner/product `<select>`s: id + names only. Avoids the full
/// `all_products` payload — notably the `effort` field, which the API computes
/// by aggregating each product's work server-side.
#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/products/product_options.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct ProductOptions;

pub async fn all_product_options(bearer: String, api_url: &str, client: Arc<Client>) -> Result<product_options::ResponseData, ApiError> {
    post_graphql::<ProductOptions>(&client, api_url, &bearer, product_options::Variables {}).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/analytics/delivery_treemap.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct DeliveryTreemap;

pub async fn delivery_treemap(bearer: String, api_url: &str, client: Arc<Client>) -> Result<delivery_treemap::ResponseData, ApiError> {
    post_graphql::<DeliveryTreemap>(&client, api_url, &bearer, delivery_treemap::Variables {}).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/products/create_product.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct CreateProduct;

pub async fn create_product(data: create_product::NewProduct, bearer: String, api_url: &str, client: Arc<Client>) -> Result<create_product::ResponseData, ApiError> {
    post_graphql::<CreateProduct>(&client, api_url, &bearer, create_product::Variables { data }).await
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(schema_path = "schema.graphql", query_path = "queries/products/update_product.graphql", response_derives = "Debug, Serialize, PartialEq")]
pub struct UpdateProduct;

pub async fn update_product(data: update_product::ProductData, bearer: String, api_url: &str, client: Arc<Client>) -> Result<update_product::ResponseData, ApiError> {
    post_graphql::<UpdateProduct>(&client, api_url, &bearer, update_product::Variables { data }).await
}
