use crate::{
    operations::{Get, List, PluginResult},
    resources::{
        utils,
        utils::{optional_cell, CreateRow, GetHeaderRow},
        DrPairId, Error,
    },
    rest_wrapper::RestClient,
};
use async_trait::async_trait;
use prettytable::Row;

/// Dr cluster pairs resource.
#[derive(clap::Args, Debug)]
pub struct DrPairs {}

/// Dr cluster pair resource.
#[derive(clap::Args, Debug)]
pub struct DrPair {}

/// Arguments used when getting a dr cluster pair.
#[derive(Debug, Clone, clap::Args)]
pub struct GetDrPairArgs {
    /// Id of the dr cluster pair.
    pub(crate) pair_id: DrPairId,
}

// CreateRow being implemented for DrClusterPair creates the rows from the pairs
// returned from the REST call.
impl CreateRow for openapi::models::DrClusterPair {
    fn row(&self) -> Row {
        // S3 is the only transport, so the store columns are its endpoint, bucket and
        // prefix. A transport that is not an object store would want its own columns
        // rather than borrowing these.
        let s3 = self.transport.s3.as_ref();
        row![
            self.id,
            self.cluster,
            // Absent until the peer answers the handshake, which is worth showing as
            // such rather than as an empty string.
            optional_cell(self.cluster_uid.as_ref()),
            self.peer,
            optional_cell(s3.map(|s3| &s3.endpoint)),
            optional_cell(s3.map(|s3| &s3.bucket)),
            optional_cell(s3.and_then(|s3| s3.prefix.as_ref())),
        ]
    }
}

impl GetHeaderRow for openapi::models::DrClusterPair {
    fn get_header_row(&self) -> Row {
        utils::DR_PAIR_HEADERS.clone()
    }
}

#[async_trait(?Send)]
impl List for DrPairs {
    async fn list(output: &utils::OutputFormat) -> PluginResult {
        match RestClient::client().dr_pairs_api().get_dr_pairs().await {
            Ok(pairs) => {
                utils::print_table(output, pairs.into_body());
                Ok(())
            }
            Err(source) => Err(Error::ListDrPairsError { source }),
        }
    }
}

#[async_trait(?Send)]
impl Get for DrPair {
    type ID = DrPairId;
    async fn get(id: &Self::ID, output: &utils::OutputFormat) -> PluginResult {
        match RestClient::client().dr_pairs_api().get_dr_pair(id).await {
            Ok(pair) => {
                utils::print_table(output, pair.into_body());
                Ok(())
            }
            Err(source) => Err(Error::GetDrPairError {
                id: id.to_owned(),
                source,
            }),
        }
    }
}
