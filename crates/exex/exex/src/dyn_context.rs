//! Mirrored version of [`ExExContext`](`crate::ExExContext`)
//! without generic abstraction over [<Node>](`reth_node_api::FullNodeComponents`)

use reth_chainspec::{EthChainSpec, Head};
use reth_node_api::FullNodeComponents;
use reth_node_core::node_config::NodeConfig;
use tokio::sync::mpsc;

use crate::{ExExContext, ExExEvent};

// TODO(0xurb) - add `node` and `notifications` after abstractions
/// Captures the context that an `ExEx` has access to.
#[derive(Debug)]
pub struct ExExContextDyn {
    /// The current head of the blockchain at launch.
    pub head: Head,
    /// The config of the node
    pub config: NodeConfig<Box<dyn EthChainSpec + 'static>>,
    /// The loaded node config
    pub reth_config: reth_config::Config,
    /// Channel used to send [`ExExEvent`]s to the rest of the node.
    ///
    /// # Important
    ///
    /// The exex should emit a `FinishedHeight` whenever a processed block is safe to prune.
    /// Additionally, the exex can pre-emptively emit a `FinishedHeight` event to specify what
    /// blocks to receive notifications for.
    pub events: mpsc::UnboundedSender<ExExEvent>,
}

impl<Node: FullNodeComponents> From<ExExContext<Node>> for ExExContextDyn {
    fn from(ctx: ExExContext<Node>) -> Self {
        Self {
            head: ctx.head,
            config: ctx.config.into_dyn(),
            reth_config: ctx.reth_config,
            events: ctx.events,
        }
    }
}
