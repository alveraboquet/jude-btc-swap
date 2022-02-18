use crate::bitcoin::{self};
use crate::judecoin;
use crate::protocol::alice::AliceState;
use crate::protocol::Database;
use anyhow::{bail, Result};
use libp2p::PeerId;
use std::convert::TryInto;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "Counterparty {0} did not refund the BTC yet. You can try again later or try to punish."
    )]
    RefundTransactionNotPublishedYet(PeerId),

    // Errors indicating that the swap cannot be refunded because because it is in a abort/final
    // state
    #[error("Swap is in state {0} where no JUDE was locked. Try aborting instead.")]
    NoXmrLocked(AliceState),
    #[error("Swap is in state {0} which is not refundable")]
    SwapNotRefundable(AliceState),
}

pub async fn refund(
    swap_id: Uuid,
    bitcoin_wallet: Arc<bitcoin::Wallet>,
    judecoin_wallet: Arc<judecoin::Wallet>,
    db: Arc<dyn Database>,
) -> Result<AliceState> {
    let state = db.get_state(swap_id).await?.try_into()?;

    let (judecoin_wallet_restore_blockheight, transfer_proof, state3) = match state {
        // In case no JUDE has been locked, move to Safely Aborted
        AliceState::Started { .. }
        | AliceState::BtcLockTransactionSeen { .. }
        | AliceState::BtcLocked { .. } => bail!(Error::NoXmrLocked(state)),

        // Refund potentially possible (no knowledge of cancel transaction)
        AliceState::XmrLockTransactionSent { judecoin_wallet_restore_blockheight, transfer_proof, state3, }
        | AliceState::XmrLocked { judecoin_wallet_restore_blockheight, transfer_proof, state3 }
        | AliceState::XmrLockTransferProofSent { judecoin_wallet_restore_blockheight, transfer_proof, state3 }
        | AliceState::EncSigLearned { judecoin_wallet_restore_blockheight, transfer_proof, state3, .. }
        | AliceState::CancelTimelockExpired { judecoin_wallet_restore_blockheight, transfer_proof, state3 }

        // Refund possible due to cancel transaction already being published
        | AliceState::BtcCancelled { judecoin_wallet_restore_blockheight, transfer_proof, state3 }
        | AliceState::BtcRefunded { judecoin_wallet_restore_blockheight, transfer_proof, state3, .. }
        | AliceState::BtcPunishable { judecoin_wallet_restore_blockheight, transfer_proof, state3, .. } => {
            (judecoin_wallet_restore_blockheight, transfer_proof, state3)
        }

        // Alice already in final state
        AliceState::BtcRedeemTransactionPublished { .. }
        | AliceState::BtcRedeemed
        | AliceState::XmrRefunded
        | AliceState::BtcPunished
        | AliceState::SafelyAborted => bail!(Error::SwapNotRefundable(state)),
    };

    tracing::info!(%swap_id, "Trying to manually refund swap");

    let spend_key = if let Ok(published_refund_tx) =
        state3.fetch_tx_refund(bitcoin_wallet.as_ref()).await
    {
        tracing::debug!(%swap_id, "Bitcoin refund transaction found, extracting key to refund judecoin");
        state3.extract_judecoin_private_key(published_refund_tx)?
    } else {
        let bob_peer_id = db.get_peer_id(swap_id).await?;
        bail!(Error::RefundTransactionNotPublishedYet(bob_peer_id),);
    };

    state3
        .refund_jude(
            &judecoin_wallet,
            judecoin_wallet_restore_blockheight,
            swap_id.to_string(),
            spend_key,
            transfer_proof,
        )
        .await?;

    let state = AliceState::XmrRefunded;
    db.insert_latest_state(swap_id, state.clone().into())
        .await?;

    Ok(state)
}
