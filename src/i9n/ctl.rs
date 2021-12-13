// LNP Node: node running lightning network protocol and generalized lightning
// channels.
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use bitcoin::{OutPoint, Txid};
use bitcoin_onchain::blockchain::MiningStatus;
use internet2::NodeAddr;
use lnp::bolt::{CommonParams, Keyset, PeerParams, Policy};
use lnp::p2p::legacy::{ChannelId, OpenChannel};
use psbt::Psbt;
#[cfg(feature = "rgb")]
use rgb::Consignment;
use strict_encoding::{NetworkDecode, NetworkEncode};
use wallet::address::AddressCompat;

use crate::i9n::rpc::{ChannelInfo, Failure, OptionDetails, PeerInfo};
use crate::service::ClientId;
use crate::ServiceId;

/// RPC API requests over CTL message bus between LNP Node daemons and from/to clients.
#[derive(Clone, Debug, Display, From)]
#[derive(NetworkEncode, NetworkDecode)]
#[non_exhaustive]
pub enum CtlMsg {
    #[display("hello()")]
    Hello,

    #[display("update_channel_id({0})")]
    UpdateChannelId(ChannelId),

    // Node connectivity API
    // ---------------------
    // Sent from lnpd to peerd
    #[display("get_info()")]
    GetInfo,

    #[display("ping_peer()")]
    PingPeer,

    // Channel creation API
    // --------------------
    /// Initiates creation of a new channel by a local node. Sent from lnpd to a newly instantiated
    /// channeld.
    #[display("open_channel_with({0})")]
    OpenChannelWith(OpenChannelWith),

    /// Initiates acceptance of a new channel proposed by a remote node. Sent from lnpd to a newly
    /// instantiated channeld.
    #[display("accept_channel_from({0})")]
    AcceptChannelFrom(AcceptChannelFrom),

    /// Constructs funding PSBT to fund a locally-created new channel. Sent from peerd to lnpd.
    #[display("construct_funding({0})")]
    ConstructFunding(FundChannel),

    /// Provides channeld with the information about funding transaction output used to fund the
    /// newly created channel. Sent from lnpd to channeld.
    #[display("funding_constructed({0})")]
    FundingConstructed(OutPoint),

    /// Signs previously prepared funding transaction and publishes it to bitcoin network. Sent
    /// from channeld to lnpd upon receival of `funding_signed` message from a remote peer.
    #[display("publish_funding()")]
    PublishFunding,

    /// Reports back to channeld that the funding transaction was published and its mining status
    /// should be monitored onchain.
    #[display("funding_published()")]
    FundingPublished,

    // On-chain tracking API
    // ---------------------
    /// Asks on-chain tracking service to send updates on the transaction mining status
    #[display("track({0})")]
    Track(Txid),

    /// Asks on-chain tracking service to stop sending updates on the transaction mining status
    #[display("untrack({0})")]
    Untrack(Txid),

    /// Reports changes in the mining status for previously requested transaction tracked by an
    /// on-chain service
    #[display("mined({0})")]
    Mined(MiningInfo),

    #[display("sign(...)")]
    Sign(Psbt),

    #[display("signed(...)")]
    Signed(Psbt),

    // Responses
    // ---------
    #[display("progress(\"{0}\")")]
    #[from]
    Report(Report),

    #[display("node_info({0})", alt = "{0:#}")]
    PeerInfo(PeerInfo),

    #[display("channel_info({0})", alt = "{0:#}")]
    ChannelInfo(ChannelInfo),
}

/// Request configuring newly launched channeld instance
#[derive(Clone, PartialEq, Eq, Debug, Display, NetworkEncode, NetworkDecode)]
#[display("{remote_peer}, {funding_sat}, ...")]
pub struct OpenChannelWith {
    /// Node to open a channel with
    pub remote_peer: NodeAddr,

    /// Client identifier to report about the progress
    pub report_to: Option<ServiceId>,

    /// Amount of satoshis for channel funding
    pub funding_sat: u64,

    /// Amount of millisatoshis to pay to the remote peer at the channel opening
    pub push_msat: u64,

    /// Channel policies
    pub policy: Policy,

    /// Channel common parameters
    pub common_params: CommonParams,

    /// Channel local parameters
    pub local_params: PeerParams,

    /// Channel local keyset
    pub local_keys: Keyset,
}

/// Request configuring newly launched channeld instance
#[derive(Clone, PartialEq, Eq, Debug, Display, NetworkEncode, NetworkDecode)]
#[display("{remote_peer}, {channel_req}, ...")]
pub struct AcceptChannelFrom {
    /// Node to open a channel with
    pub remote_peer: NodeAddr,

    /// Client identifier to report about the progress
    pub report_to: Option<ServiceId>,

    /// Request received from a remote peer to open channel
    pub channel_req: OpenChannel,

    /// Channel policies
    pub policy: Policy,

    /// Channel common parameters
    pub common_params: CommonParams,

    /// Channel local parameters
    pub local_params: PeerParams,

    /// Channel local keyset
    pub local_keys: Keyset,
}

/// Request information about constructing funding transaction
#[derive(Clone, PartialEq, Eq, Debug, Display, NetworkEncode, NetworkDecode)]
#[display("{address}, {amount}")]
pub struct FundChannel {
    /// Address for the channel funding
    pub address: AddressCompat,

    /// Amount of funds to be sent to the funding address
    pub amount: u64,

    /// Fee to pay for the funding transaction
    pub fee: u64,
}

/// Update on a transaction mining status
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[derive(NetworkEncode, NetworkDecode)]
#[display("{txid}, {status}")]
pub struct MiningInfo {
    /// Id of a transaction previously requested to be tracked
    pub txid: Txid,

    /// Updated on-chain status of the transaction
    pub status: MiningStatus,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, NetworkEncode, NetworkDecode)]
#[display("{client}, {status}")]
pub struct Report {
    pub client: ClientId,
    pub status: Status,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, From)]
#[derive(NetworkEncode, NetworkDecode)]
pub enum Status {
    #[display("progress = \"{0}\"")]
    #[from]
    Progress(String),

    #[display("success = {0}")]
    Success(OptionDetails),

    #[display("failure = {0}")]
    #[from]
    Failure(Failure),
}
