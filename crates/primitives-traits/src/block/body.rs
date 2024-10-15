//! Block body abstraction

use alloc::fmt;
use alloy_consensus::{BlockHeader, Transaction, TxType};
use revm_primitives::{Address, B256};

use crate::{Requests, Withdrawals};

use super::Block;

/// Abstraction for block's body.
pub trait BlockBody:
    Clone
    + fmt::Debug
    + PartialEq
    + Eq
    + Default
    + serde::Serialize
    + for<'de> serde::Deserialize<'de>
    + alloy_rlp::Encodable
    + alloy_rlp::Decodable
{
    /// Ordered list of signed transactions as committed in block.
    // todo: requires trait for signed transaction
    type SignedTransaction: Transaction;

    /// Header type (uncle blocks).
    type Header: BlockHeader;

    /// Returns reference to transactions in block.
    fn transactions(&self) -> &[Self::SignedTransaction];

    /// Returns [`Withdrawals`] in the block, if any.
    // todo: branch out into extension trait
    fn withdrawals(&self) -> Option<&Withdrawals>;

    /// Returns reference to uncle block headers.
    fn ommers(&self) -> &[Self::Header];

    /// Returns [`Request`] in block, if any.
    fn requests(&self) -> Option<&Requests>;

    /// Create a [`Block`] from the body and its header.
    fn into_block<T: Block<Header = Self::Header, Body = Self>>(self, header: Self::Header) -> T {
        T::from((header, self))
    }

    /// Calculate the transaction root for the block body.
    fn calculate_tx_root(&self) -> B256;

    /// Calculate the ommers root for the block body.
    fn calculate_ommers_root(&self) -> B256;

    /// Recover signer addresses for all transactions in the block body.
    fn recover_signers(&self) -> Option<Vec<Address>>;

    /// Returns whether or not the block body contains any blob transactions.
    fn has_blob_transactions(&self) -> bool {
        self.transactions().iter().any(|tx| tx.ty() as u8 == TxType::Eip4844 as u8)
    }

    /// Returns whether or not the block body contains any EIP-7702 transactions.
    fn has_eip7702_transactions(&self) -> bool {
        self.transactions().iter().any(|tx| tx.ty() as u8 == TxType::Eip7702 as u8)
    }

    /// Returns an iterator over all blob transactions of the block
    fn blob_transactions_iter(&self) -> impl Iterator<Item = &Self::SignedTransaction> + '_ {
        self.transactions().iter().filter(|tx| tx.ty() as u8 == TxType::Eip4844 as u8)
    }

    /// Returns only the blob transactions, if any, from the block body.
    fn blob_transactions(&self) -> Vec<&Self::SignedTransaction> {
        self.blob_transactions_iter().collect()
    }

    /// Returns an iterator over all blob versioned hashes from the block body.
    fn blob_versioned_hashes_iter(&self) -> impl Iterator<Item = &B256> + '_;

    /// Returns all blob versioned hashes from the block body.
    fn blob_versioned_hashes(&self) -> Vec<&B256> {
        self.blob_versioned_hashes_iter().collect()
    }

    /// Calculates a heuristic for the in-memory size of the [`BlockBody`].
    fn size(&self) -> usize;
}
