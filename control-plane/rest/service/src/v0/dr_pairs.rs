use super::*;
use grpc::operations::dr_pair::traits::DrPairOperations;
use stor_port::types::v0::transport::{
    CreateDrClusterPair, DestroyDrClusterPair, DrClusterPair, DrDiscovery, DrS3Transport,
    DrTransport, GetDrClusterPair,
};

fn client() -> impl DrPairOperations {
    core_grpc().dr_pair()
}

/// The transport as the API expresses it. A body that names no store has nowhere for the
/// relationship to live, which there is no sensible default for.
fn transport(src: models::DrTransport) -> Result<DrTransport, ReplyError> {
    match src.s3 {
        Some(s3) => Ok(DrTransport::S3(DrS3Transport {
            endpoint: s3.endpoint,
            bucket: s3.bucket,
            prefix: s3.prefix,
            credentials_secret: s3.credentials_secret,
            collector_credentials_secret: s3.collector_credentials_secret,
        })),
        None => Err(ReplyError::missing_argument(
            ResourceKind::DrClusterPair,
            "transport.s3",
        )),
    }
}

/// An absent envelope bounds nothing, which is the same as an empty one.
fn discovery(src: Option<models::DrDiscovery>) -> DrDiscovery {
    match src {
        Some(discovery) => DrDiscovery {
            namespace: discovery.namespace,
            storage_class: discovery.storage_class,
            max_volume_size: discovery.max_volume_size.map(|size| size as u64),
        },
        None => DrDiscovery::default(),
    }
}

fn to_models(pair: DrClusterPair) -> models::DrClusterPair {
    let DrTransport::S3(s3) = pair.transport;
    models::DrClusterPair::new_all(
        pair.id.to_string(),
        pair.cluster.to_string(),
        pair.cluster_uid.map(|uid| uid.to_string()),
        pair.peer.to_string(),
        models::DrTransport::new_all(Some(models::DrS3Transport::new_all(
            s3.endpoint,
            s3.bucket,
            s3.prefix,
            s3.credentials_secret,
            s3.collector_credentials_secret,
        ))),
        Some(models::DrDiscovery::new_all(
            pair.discovery.namespace,
            pair.discovery.storage_class,
            pair.discovery.max_volume_size.map(|size| size as i64),
        )),
    )
}

#[async_trait::async_trait]
impl apis::actix_server::DrPairs for RestApi {
    async fn del_dr_pair(Path(pair_id): Path<String>) -> Result<(), RestError<RestJsonError>> {
        client()
            .destroy(&DestroyDrClusterPair { id: pair_id.into() }, None)
            .await?;
        Ok(())
    }

    async fn get_dr_pair(
        Path(pair_id): Path<String>,
    ) -> Result<models::DrClusterPair, RestError<RestJsonError>> {
        let pair = client()
            .get(&GetDrClusterPair { id: pair_id.into() }, None)
            .await?;
        Ok(to_models(pair))
    }

    async fn get_dr_pairs() -> Result<Vec<models::DrClusterPair>, RestError<RestJsonError>> {
        let pairs = client().list(None).await?;
        Ok(pairs.entries.into_iter().map(to_models).collect())
    }

    async fn put_dr_pair(
        Path(pair_id): Path<String>,
        Body(body): Body<models::CreateDrClusterPairBody>,
    ) -> Result<models::DrClusterPair, RestError<RestJsonError>> {
        let pair = client()
            .create(
                &CreateDrClusterPair {
                    id: pair_id.into(),
                    cluster: body.cluster.into(),
                    peer: body.peer.into(),
                    transport: transport(body.transport)?,
                    discovery: discovery(body.discovery),
                },
                None,
            )
            .await?;
        Ok(to_models(pair))
    }
}
