//! Run an JUDE/BTC swap in the role of Alice.
//! Alice holds JUDE and wishes receive BTC.
use crate::env::Config;
use crate::protocol::Database;
use crate::{asb, bitcoin, judecoin};
use std::sync::Arc;
use uuid::Uuid;

pub use self::state::*;
pub use self::swap::{run, run_until};

pub mod state;
pub mod swap;

pub struct Swap {
    pub state: AliceState,
    pub event_loop_handle: asb::EventLoopHandle,
    pub bitcoin_wallet: Arc<bitcoin::Wallet>,
    pub judecoin_wallet: Arc<judecoin::Wallet>,
    pub env_config: Config,
    pub swap_id: Uuid,
    pub db: Arc<dyn Database + Send + Sync>,
}
