#![allow(unused_imports)]

use crate::bq;
use anyhow::Result;
use ethers::types::H160;
use gcp_bigquery_client::model::{
    dataset::Dataset, query_request::QueryRequest, range_partitioning::RangePartitioning,
    range_partitioning_range::RangePartitioningRange, table::Table,
    table_data_insert_all_request::TableDataInsertAllRequest, table_field_schema::TableFieldSchema,
    table_schema::TableSchema,
};
use gcp_bigquery_client::Client;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

#[derive(Serialize, Deserialize)]
pub struct InvolvementResult {
    txn_id: Option<String>,
    data: Option<String>,
    code: Option<String>,
    receipt: Option<String>,
    to_addr: Option<String>,
    from_addr_zil: Option<String>,
}

pub async fn query_involved(bq: &bq::BQ, address_val: &H160) -> Result<Vec<InvolvementResult>> {
    let table = bq.loc.table_name("transactions");
    let address = format!("{0:#020x}", &address_val);
    let query = format!("SELECT id,SAFE_CONVERT_BYTES_TO_STRING(data), SAFE_CONVERT_BYTES_TO_STRING(code),receipt,to_addr, from_addr_zil FROM `{table}` WHERE SAFE_CONVERT_BYTES_TO_STRING(data) LIKE '%{address}%' OR from_addr_zil = '0x{address}' OR receipt LIKE '%{address}%' OR SAFE_CONVERT_BYTES_TO_STRING(code) LIKE '%{address}%' ORDER BY block,offset_in_block ASC");
    let mut result = bq
        .bq_client
        .job()
        .query(&bq.loc.project_id, QueryRequest::new(&query))
        .await?;
    let mut output: Vec<InvolvementResult> = Vec::new();
    while result.next_row() {
        let current = InvolvementResult {
            txn_id: result.get_string(0)?,
            data: result.get_string(1)?,
            code: result.get_string(2)?,
            receipt: result.get_string(3)?,
            to_addr: result.get_string(4)?,
            from_addr_zil: result.get_string(5)?,
        };
        output.push(current);
    }
    Ok(output)
}
