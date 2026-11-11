use crate::types::v0::{
    store::SpecStatus,
    transport::{DrClusterId, DrClusterUid, DrDiscovery, DrPairId, DrTransport},
};
use pstor::{ApiVersion, ObjectKey, StorableObject, StorableObjectType};
use serde::{Deserialize, Serialize};

/// Key used by the store to uniquely identify a DrClusterPairSpec structure.
pub struct DrClusterPairSpecKey(DrPairId);

impl From<&DrPairId> for DrClusterPairSpecKey {
    fn from(id: &DrPairId) -> Self {
        Self(id.clone())
    }
}

impl ObjectKey for DrClusterPairSpecKey {
    type Kind = StorableObjectType;

    fn version(&self) -> ApiVersion {
        ApiVersion::V0
    }

    fn key_type(&self) -> StorableObjectType {
        StorableObjectType::DrClusterPairSpec
    }

    fn key_uuid(&self) -> String {
        self.0.to_string()
    }
}

impl StorableObject for DrClusterPairSpec {
    type Key = DrClusterPairSpecKey;

    fn key(&self) -> Self::Key {
        DrClusterPairSpecKey(self.id.clone())
    }
}

/// Lifecycle of a cluster pair, as for any other spec.
pub type DrClusterPairSpecStatus = SpecStatus<()>;

/// The pairing of this cluster with a peer, and the store they share.
///
/// Persisted separately from the per-volume dr entry because the two have different
/// lifetimes: a pair outlives every relationship underneath it, and many volumes name the
/// same one. It is also where the location of everything else comes from - the bucket and
/// prefix for every key this side writes, and the name it expects to find the peer under.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DrClusterPairSpec {
    /// How volumes refer to this pair.
    pub id: DrPairId,
    /// This cluster's name in the store.
    pub cluster: DrClusterId,
    /// This cluster's own identity, filled in by the agent rather than configured.
    ///
    /// Absent until the agent has read it from the control plane, which is the honest
    /// answer rather than an empty string that reads as a name nobody chose.
    pub cluster_uid: Option<DrClusterUid>,
    /// The peer's name in the store.
    pub peer: DrClusterId,
    /// Where the store is.
    pub transport: DrTransport,
    /// What a destination may auto-create under this pair.
    pub discovery: DrDiscovery,
    /// Lifecycle of this entry.
    pub status: DrClusterPairSpecStatus,
}

impl DrClusterPairSpec {
    /// Returns a new cluster pair in its creating state.
    pub fn new(
        id: DrPairId,
        cluster: DrClusterId,
        peer: DrClusterId,
        transport: DrTransport,
        discovery: DrDiscovery,
    ) -> Self {
        Self {
            id,
            cluster,
            cluster_uid: None,
            peer,
            transport,
            discovery,
            status: DrClusterPairSpecStatus::Creating,
        }
    }
    /// How volumes refer to this pair.
    pub fn id(&self) -> &DrPairId {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::v0::transport::DrS3Transport;

    fn pair() -> DrClusterPairSpec {
        DrClusterPairSpec::new(
            "dr-west".into(),
            "east".into(),
            "west".into(),
            DrTransport::S3(DrS3Transport {
                endpoint: "http://localhost:3900".into(),
                bucket: "default".into(),
                prefix: Some("prod".into()),
                credentials_secret: None,
                collector_credentials_secret: None,
            }),
            DrDiscovery::default(),
        )
    }

    #[test]
    fn dr_cluster_pair_round_trips() {
        let spec = pair();
        let json = serde_json::to_string(&spec).unwrap();
        let back: DrClusterPairSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(spec, back);
    }

    #[test]
    fn dr_cluster_pair_key_is_the_pair_id() {
        let spec = pair();
        assert_eq!(spec.key().key_uuid(), spec.id.to_string());
        assert_eq!(
            spec.key().key_type().to_string(),
            StorableObjectType::DrClusterPairSpec.to_string()
        );
    }

    #[test]
    fn cluster_uid_starts_absent() {
        // The agent fills it from the control plane's own identity; it is not the
        // administrator's to choose, so a new pair has none rather than an empty one.
        assert!(pair().cluster_uid.is_none());
    }
}
