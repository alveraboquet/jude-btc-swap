use crate::bitcoin::EncryptedSignature;
use crate::judecoin;
use crate::judecoin::{judecoin_private_key, TransferProof};
use crate::protocol::alice;
use crate::protocol::alice::AliceState;
use judecoin_rpc::wallet::BlockHeight;
use serde::{Deserialize, Serialize};
use std::fmt;

// Large enum variant is fine because this is only used for database
// and is dropped once written in DB.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Alice {
    Started {
        state3: alice::State3,
    },
    BtcLockTransactionSeen {
        state3: alice::State3,
    },
    BtcLocked {
        state3: alice::State3,
    },
    XmrLockTransactionSent {
        judecoin_wallet_restore_blockheight: BlockHeight,
        transfer_proof: TransferProof,
        state3: alice::State3,
    },
    XmrLocked {
        judecoin_wallet_restore_blockheight: BlockHeight,
        transfer_proof: TransferProof,
        state3: alice::State3,
    },
    XmrLockTransferProofSent {
        judecoin_wallet_restore_blockheight: BlockHeight,
        transfer_proof: TransferProof,
        state3: alice::State3,
    },
    EncSigLearned {
        judecoin_wallet_restore_blockheight: BlockHeight,
        transfer_proof: TransferProof,
        encrypted_signature: EncryptedSignature,
        state3: alice::State3,
    },
    BtcRedeemTransactionPublished {
        state3: alice::State3,
    },
    CancelTimelockExpired {
        judecoin_wallet_restore_blockheight: BlockHeight,
        transfer_proof: TransferProof,
        state3: alice::State3,
    },
    BtcCancelled {
        judecoin_wallet_restore_blockheight: BlockHeight,
        transfer_proof: TransferProof,
        state3: alice::State3,
    },
    BtcPunishable {
        judecoin_wallet_restore_blockheight: BlockHeight,
        transfer_proof: TransferProof,
        state3: alice::State3,
    },
    BtcRefunded {
        judecoin_wallet_restore_blockheight: BlockHeight,
        transfer_proof: TransferProof,
        state3: alice::State3,
        #[serde(with = "judecoin_private_key")]
        spend_key: judecoin::PrivateKey,
    },
    Done(AliceEndState),
}

#[derive(Copy, Clone, strum::Display, Debug, Deserialize, Serialize, PartialEq)]
pub enum AliceEndState {
    SafelyAborted,
    BtcRedeemed,
    XmrRefunded,
    BtcPunished,
}

impl From<AliceState> for Alice {
    fn from(alice_state: AliceState) -> Self {
        match alice_state {
            AliceState::Started { state3 } => Alice::Started {
                state3: state3.as_ref().clone(),
            },
            AliceState::BtcLockTransactionSeen { state3 } => Alice::BtcLockTransactionSeen {
                state3: state3.as_ref().clone(),
            },
            AliceState::BtcLocked { state3 } => Alice::BtcLocked {
                state3: state3.as_ref().clone(),
            },
            AliceState::XmrLockTransactionSent {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3,
            } => Alice::XmrLockTransactionSent {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3: state3.as_ref().clone(),
            },
            AliceState::XmrLocked {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3,
            } => Alice::XmrLocked {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3: state3.as_ref().clone(),
            },
            AliceState::XmrLockTransferProofSent {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3,
            } => Alice::XmrLockTransferProofSent {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3: state3.as_ref().clone(),
            },
            AliceState::EncSigLearned {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3,
                encrypted_signature,
            } => Alice::EncSigLearned {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3: state3.as_ref().clone(),
                encrypted_signature: encrypted_signature.as_ref().clone(),
            },
            AliceState::BtcRedeemTransactionPublished { state3 } => {
                Alice::BtcRedeemTransactionPublished {
                    state3: state3.as_ref().clone(),
                }
            }
            AliceState::BtcRedeemed => Alice::Done(AliceEndState::BtcRedeemed),
            AliceState::BtcCancelled {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3,
            } => Alice::BtcCancelled {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3: state3.as_ref().clone(),
            },
            AliceState::BtcRefunded {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                spend_key,
                state3,
            } => Alice::BtcRefunded {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                spend_key,
                state3: state3.as_ref().clone(),
            },
            AliceState::BtcPunishable {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3,
            } => Alice::BtcPunishable {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3: state3.as_ref().clone(),
            },
            AliceState::XmrRefunded => Alice::Done(AliceEndState::XmrRefunded),
            AliceState::CancelTimelockExpired {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3,
            } => Alice::CancelTimelockExpired {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3: state3.as_ref().clone(),
            },
            AliceState::BtcPunished => Alice::Done(AliceEndState::BtcPunished),
            AliceState::SafelyAborted => Alice::Done(AliceEndState::SafelyAborted),
        }
    }
}

impl From<Alice> for AliceState {
    fn from(db_state: Alice) -> Self {
        match db_state {
            Alice::Started { state3 } => AliceState::Started {
                state3: Box::new(state3),
            },
            Alice::BtcLockTransactionSeen { state3 } => AliceState::BtcLockTransactionSeen {
                state3: Box::new(state3),
            },
            Alice::BtcLocked { state3 } => AliceState::BtcLocked {
                state3: Box::new(state3),
            },
            Alice::XmrLockTransactionSent {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3,
            } => AliceState::XmrLockTransactionSent {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3: Box::new(state3),
            },
            Alice::XmrLocked {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3,
            } => AliceState::XmrLocked {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3: Box::new(state3),
            },
            Alice::XmrLockTransferProofSent {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3,
            } => AliceState::XmrLockTransferProofSent {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3: Box::new(state3),
            },
            Alice::EncSigLearned {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3: state,
                encrypted_signature,
            } => AliceState::EncSigLearned {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3: Box::new(state),
                encrypted_signature: Box::new(encrypted_signature),
            },
            Alice::BtcRedeemTransactionPublished { state3 } => {
                AliceState::BtcRedeemTransactionPublished {
                    state3: Box::new(state3),
                }
            }
            Alice::CancelTimelockExpired {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3,
            } => AliceState::CancelTimelockExpired {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3: Box::new(state3),
            },
            Alice::BtcCancelled {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3,
            } => AliceState::BtcCancelled {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3: Box::new(state3),
            },

            Alice::BtcPunishable {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3,
            } => AliceState::BtcPunishable {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                state3: Box::new(state3),
            },
            Alice::BtcRefunded {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                spend_key,
                state3,
            } => AliceState::BtcRefunded {
                judecoin_wallet_restore_blockheight,
                transfer_proof,
                spend_key,
                state3: Box::new(state3),
            },
            Alice::Done(end_state) => match end_state {
                AliceEndState::SafelyAborted => AliceState::SafelyAborted,
                AliceEndState::BtcRedeemed => AliceState::BtcRedeemed,
                AliceEndState::XmrRefunded => AliceState::XmrRefunded,
                AliceEndState::BtcPunished => AliceState::BtcPunished,
            },
        }
    }
}

impl fmt::Display for Alice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Alice::Started { .. } => write!(f, "Started"),
            Alice::BtcLockTransactionSeen { .. } => {
                write!(f, "Bitcoin lock transaction in mempool")
            }
            Alice::BtcLocked { .. } => f.write_str("Bitcoin locked"),
            Alice::XmrLockTransactionSent { .. } => f.write_str("judecoin lock transaction sent"),
            Alice::XmrLocked { .. } => f.write_str("judecoin locked"),
            Alice::XmrLockTransferProofSent { .. } => {
                f.write_str("judecoin lock transfer proof sent")
            }
            Alice::EncSigLearned { .. } => f.write_str("Encrypted signature learned"),
            Alice::BtcRedeemTransactionPublished { .. } => {
                f.write_str("Bitcoin redeem transaction published")
            }
            Alice::CancelTimelockExpired { .. } => f.write_str("Cancel timelock is expired"),
            Alice::BtcCancelled { .. } => f.write_str("Bitcoin cancel transaction published"),
            Alice::BtcPunishable { .. } => f.write_str("Bitcoin punishable"),
            Alice::BtcRefunded { .. } => f.write_str("judecoin refundable"),
            Alice::Done(end_state) => write!(f, "Done: {}", end_state),
        }
    }
}
