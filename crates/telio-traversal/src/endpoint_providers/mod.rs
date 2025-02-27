pub mod local;
pub mod stun;
pub mod upnp;

use async_trait::async_trait;
use enum_map::Enum;
use ipnet::PrefixLenError;
use std::time::Duration;
use telio_crypto::{encryption, PublicKey};
use telio_utils::exponential_backoff;
use thiserror::Error as TError;

use telio_model::SocketAddr;
use telio_proto::{PlaintextPongerMsg, Session};
use telio_task::io::chan;
use telio_wg;

#[derive(Debug, TError)]
pub enum Error {
    #[error(transparent)]
    PrefixLenError(#[from] PrefixLenError),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    EndpointCandidatePublishError(
        #[from] tokio::sync::mpsc::error::SendError<EndpointCandidatesChangeEvent>,
    ),
    #[error(transparent)]
    ExponentialBackoffError(#[from] exponential_backoff::Error),
    #[error(transparent)]
    PongRxedPublishError(#[from] tokio::sync::mpsc::error::SendError<PongEvent>),
    #[error(transparent)]
    WireGuardError(#[from] telio_wg::Error),
    #[error(transparent)]
    AddressParseError(#[from] std::net::AddrParseError),
    #[error("WireGuard listening port is missing")]
    NoWGListenPort,
    #[error(transparent)]
    PacketParserError(#[from] telio_proto::CodecError),
    #[error(transparent)]
    UpnpError(#[from] rupnp::Error),
    #[error("Missing IGD gateway")]
    NoIGDGateway,
    #[error("The existing Upnp endpoint is invalid")]
    IGDError(#[from] igd::GetGenericPortMappingEntryError),
    #[error(transparent)]
    IGDRemovePortError(#[from] igd::RemovePortError),
    #[error(transparent)]
    IGDGetExternalIpError(#[from] igd::GetExternalIpError),
    #[error(transparent)]
    IGDSearchError(#[from] igd::SearchError),
    #[error(transparent)]
    IGDAddAnyPortError(#[from] igd::AddAnyPortError),
    #[error(transparent)]
    IGDAddPortError(#[from] igd::AddPortError),
    #[error("Failed to build pong packet")]
    FailedToBuildPongPacket,
    /// Stun codec error
    #[error(transparent)]
    ByteCodecError(#[from] bytecodec::Error),
    #[error(transparent)]
    RuntimeError(#[from] telio_task::ExecError),
    /// Component was not configured for operation
    #[error("Component was not configured for operation")]
    NotConfigured,
    /// Stun peer does not exist in wireguard
    #[error("Stun peer does not exist in wireguard")]
    NoStunPeer,
    /// Stun peer is missconfigured (no allowed_ip or endpoint)
    #[error("Stun peer is misconfigured")]
    BadStunPeer,
    #[error("Encryption failed: {0}")]
    EncryptionFailed(#[from] encryption::Error),
    #[error("Cannot find endpoint in local cache")]
    LocalCacheError,
    /// Failed to get upnp service
    #[error("Failed to get upnp service")]
    FailedToGetUpnpService,
    /// Did not find matching endpoint with the IGD subnet
    #[error("No endpoint with matching subnet to IGD")]
    NoMatchingLocalEndpoint,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Enum)]
pub enum EndpointProviderType {
    LocalInterfaces,
    Stun,
    Upnp,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct EndpointCandidate {
    pub wg: SocketAddr,
    pub udp: SocketAddr,
}

pub type EndpointCandidatesChangeEvent = (EndpointProviderType, Vec<EndpointCandidate>);

#[derive(Debug, Clone)]
pub struct PongEvent {
    pub addr: SocketAddr,
    pub rtt: Duration,
    pub msg: PlaintextPongerMsg,
}

#[cfg_attr(any(test, feature = "mockall"), mockall::automock)]
#[async_trait]
pub trait EndpointProvider: Sync + Send + 'static {
    /// Endpoint providers name
    fn name(&self) -> &'static str;

    async fn subscribe_for_pong_events(&self, tx: chan::Tx<PongEvent>);
    async fn subscribe_for_endpoint_candidates_change_events(
        &self,
        tx: chan::Tx<EndpointCandidatesChangeEvent>,
    );
    async fn trigger_endpoint_candidates_discovery(&self) -> Result<(), Error>;
    async fn handle_endpoint_gone_notification(&self);

    async fn send_ping(
        &self,
        addr: SocketAddr,
        session_id: Session,
        public_key: PublicKey,
    ) -> Result<(), Error>;

    async fn get_current_endpoints(&self) -> Option<Vec<EndpointCandidate>>;
}
