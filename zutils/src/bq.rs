#![allow(unused_imports)]

use crate::utils;
use anyhow::{anyhow, Result};
use gcp_bigquery_client::model::{
    dataset::Dataset, query_request::QueryRequest, range_partitioning::RangePartitioning,
    range_partitioning_range::RangePartitioningRange, table::Table,
    table_data_insert_all_request::TableDataInsertAllRequest, table_field_schema::TableFieldSchema,
    table_schema::TableSchema,
};
use gcp_bigquery_client::Client;
use reqwest;
use serde::{Deserialize, Serialize};
use std::{env, fmt};
use tokio::time::{sleep, Duration};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Location {
    pub project_id: String,
    pub dataset_id: String,
}

impl Location {
    pub fn new(project_id: &str, dataset_id: &str) -> Self {
        Self {
            project_id: project_id.to_string(),
            dataset_id: dataset_id.to_string(),
        }
    }

    pub fn table_name(&self, table_name: &str) -> String {
        format!("{0}.{1}.{2}", self.project_id, self.dataset_id, table_name)
    }
}

pub struct BQ {
    pub loc: Location,
    pub bq_client: Client,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DefaultCredentialSource {
    EnvVar,
    DefaultSecretsFile,
    MetadataServer,
}

pub struct DefaultCredentials {
    // The client we created
    pub client: Client,
    pub source: DefaultCredentialSource,
    pub file_name: Option<String>,
}

impl fmt::Debug for DefaultCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "DefaultCredentials[ type = {0:?} ", self.source)?;
        if let Some(the_fn) = &self.file_name {
            write!(f, "file_name = {0}", the_fn)?;
        }
        write!(f, "]")
    }
}

/// Apply google's rules to generate a bigquery client:
///
///  - if GOOGLE_APPLICATION_CREDENTIALS is defined, try that first.
///  - otherwise, have a go with the default gcloud config file (~/.config/gcloud/application_default_credentials.json)
///  - if that fails, try the metadata server.
pub async fn client_from_default_credentials() -> Result<DefaultCredentials> {
    async fn try_credentials(where_from: &str) -> Result<Client> {
        Ok(Client::from_authorized_user_secret(where_from).await?)
    }
    if let Ok(val) = env::var("GOOGLE_APPLICATION_CREDENTIALS") {
        if let Ok(result) = try_credentials(&val).await {
            return Ok(DefaultCredentials {
                client: result,
                source: DefaultCredentialSource::EnvVar,
                file_name: Some(val),
            });
        }
    }
    // If we can't find your home directory, we will simply continue ..
    if let Some(home_dir) = home::home_dir() {
        let home_as_string = home_dir.into_os_string().into_string().or(Err(anyhow!(
            "Your home directory could not be represented as a string"
        )))?;
        let default_file = format!(
            "{0}/.config/gcloud/application_default_credentials.json",
            home_as_string
        );
        if let Ok(result) = try_credentials(&default_file).await {
            return Ok(DefaultCredentials {
                client: result,
                source: DefaultCredentialSource::DefaultSecretsFile,
                file_name: Some(default_file.to_string()),
            });
        }
    }
    // OK. We got here ..
    let result = Client::from_application_default_credentials().await?;
    Ok(DefaultCredentials {
        client: result,
        source: DefaultCredentialSource::MetadataServer,
        file_name: None,
    })
}

pub async fn find_table(client: &Client, loc: &Location, table_id: &str) -> Result<Option<Table>> {
    match client
        .table()
        .get(&loc.project_id, &loc.dataset_id, table_id, Option::None)
        .await
    {
        Ok(tbl) => Ok(Some(tbl)),
        _ => Ok(None),
    }
}

impl BQ {
    pub async fn new(loc: &Location) -> Result<Self> {
        let client = client_from_default_credentials().await?;
        Ok(Self {
            loc: loc.clone(),
            bq_client: client.client,
        })
    }
}
