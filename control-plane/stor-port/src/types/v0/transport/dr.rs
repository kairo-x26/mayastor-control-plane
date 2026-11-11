use crate::{rpc_impl_string_id, rpc_impl_string_id_inner};

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

rpc_impl_string_id!(DrPairId, "ID of a dr cluster pair");
rpc_impl_string_id!(DrClusterId, "Name of a cluster within a dr relationship");
rpc_impl_string_id!(
    DrClusterUid,
    "A cluster's own identity, as its control plane keys its persistent store on"
);

/// Where a relationship's store lives, and how it is reached.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum DrTransport {
    /// An S3-compatible object store both clusters can reach.
    S3(DrS3Transport),
}

/// An S3-compatible object store.
///
/// Credential *values* are deliberately absent: they are the agent's, taken from its
/// environment, and an API that accepted them would be handing secrets around in request
/// bodies for no benefit. The secret *names* below are references for whoever mounts them,
/// which is a different thing from the secrets themselves.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct DrS3Transport {
    /// Endpoint URL of the store.
    pub endpoint: String,
    /// Bucket the relationships live in.
    pub bucket: String,
    /// Key prefix within the bucket, when the bucket is shared with something else.
    pub prefix: Option<String>,
    /// Name of the secret holding the credentials the cycle and convergence paths use.
    /// That identity needs no delete on `chunks/` or `pits/`.
    pub credentials_secret: Option<String>,
    /// Name of the secret holding the collector's credentials, the only identity that
    /// deletes. Absent where nothing reclaims.
    pub collector_credentials_secret: Option<String>,
}

/// What a destination may create for itself when it meets a volume it has never seen.
///
/// Auto-creation is bounded rather than free: a relationship names a namespace, a storage
/// class and a ceiling, and a volume outside any of them is reported rather than built.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Default)]
pub struct DrDiscovery {
    /// Namespace a discovered volume may be created in.
    pub namespace: Option<String>,
    /// Storage class a discovered volume is created with.
    pub storage_class: Option<String>,
    /// Largest volume that may be created. Bigger ones are reported, not built.
    pub max_volume_size: Option<u64>,
}

/// The pairing of this cluster with a peer, and the store they share.
///
/// Cluster-scoped rather than per-volume: a volume's dr entry names a pair, and many
/// volumes name the same one. The two cluster names are the whole handshake - each side
/// writes under its own and reads the other's.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct DrClusterPair {
    /// How volumes refer to this pair.
    pub id: DrPairId,
    /// This cluster's name in the store.
    pub cluster: DrClusterId,
    /// The uid of the cluster this pair was created on, which is the one its control plane
    /// keys its own persistent store on.
    ///
    /// Not configured: a name is chosen by an administrator and two clusters may be given
    /// the same one, so this is what actually distinguishes them, and it is the second half
    /// of every store key this side writes - `<cluster>/<cluster_uid>/`.
    pub cluster_uid: Option<DrClusterUid>,
    /// The peer's name in the store.
    pub peer: DrClusterId,
    /// Where the store is.
    pub transport: DrTransport,
    /// What a destination may auto-create under this pair.
    pub discovery: DrDiscovery,
}

/// Create a cluster pair, or adopt one that already exists.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct CreateDrClusterPair {
    /// How volumes will refer to this pair.
    pub id: DrPairId,
    /// This side's name in the store for this relationship.
    ///
    /// Not a claim about which cluster this is - that is the cluster's uid, which is not
    /// chosen by hand. So a name other than this cluster's own is allowed, and is what lets
    /// one cluster hold both sides of a relationship: two pairs, `east -> west` and
    /// `west -> east`, over the same store. It must differ from `peer`.
    pub cluster: DrClusterId,
    /// The peer's name in the store.
    pub peer: DrClusterId,
    /// Where the store is.
    pub transport: DrTransport,
    /// What a destination may auto-create under this pair.
    pub discovery: DrDiscovery,
}

/// Fetch a single cluster pair.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct GetDrClusterPair {
    /// The pair to fetch.
    pub id: DrPairId,
}

/// Remove a cluster pair.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct DestroyDrClusterPair {
    /// The pair to remove.
    pub id: DrPairId,
}

/// A page of cluster pairs.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Default)]
pub struct DrClusterPairs {
    /// The pairs in this page.
    pub entries: Vec<DrClusterPair>,
    /// Token to resume from, when more remain.
    pub next_token: Option<u64>,
}
