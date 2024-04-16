use {
    solana_rpc_client_api::{client_error::Error as ClientError, config::RpcBlockConfig},
    solana_sdk::{
        account::Account,
        commitment_config::CommitmentConfig,
        epoch_info::EpochInfo,
        hash::Hash,
        message::Message,
        pubkey::Pubkey,
        signature::Signature,
        slot_history::Slot,
        transaction::{Result, Transaction},
        transport::TransportError,
    },
    solana_tpu_client::tpu_client::TpuSenderError,
    solana_transaction_status::UiConfirmedBlock,
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum GenericClientError {
    #[error("Airdrop failure")]
    AirdropFailure,
    #[error("IO error: {0:?}")]
    IoError(#[from] std::io::Error),
    #[error("Client error: {0:?}")]
    ClientError(#[from] ClientError),
    #[error("TpuClient error: {0:?}")]
    TpuSenderError(#[from] TpuSenderError),
    #[error("Transport error: {0:?}")]
    TransportError(#[from] TransportError),
    #[error("Custom error: {0}")]
    Custom(String),
}

pub(crate) type GenericClientResult<T> = std::result::Result<T, GenericClientError>;

pub trait GenericClient {
    /// Send a signed transaction without confirmation
    fn send_transaction(&self, transaction: Transaction) -> GenericClientResult<Signature>;

    /// Send a batch of signed transactions without confirmation.
    fn send_batch(&self, transactions: Vec<Transaction>) -> GenericClientResult<()>;

    /// Get latest blockhash
    fn get_latest_blockhash(&self) -> GenericClientResult<Hash>;

    /// Get latest blockhash and its last valid block height, using explicit commitment
    fn get_latest_blockhash_with_commitment(
        &self,
        commitment_config: CommitmentConfig,
    ) -> GenericClientResult<(Hash, u64)>;

    fn get_new_latest_blockhash(&self, blockhash: &Hash) -> GenericClientResult<Hash>;

    fn get_signature_status(
        &self,
        signature: &Signature,
    ) -> GenericClientResult<Option<Result<()>>>;

    /// Get transaction count
    fn get_transaction_count(&self) -> GenericClientResult<u64>;

    /// Get transaction count, using explicit commitment
    fn get_transaction_count_with_commitment(
        &self,
        commitment_config: CommitmentConfig,
    ) -> GenericClientResult<u64>;

    /// Get epoch info
    fn get_epoch_info(&self) -> GenericClientResult<EpochInfo>;

    /// Get account balance
    fn get_balance(&self, pubkey: &Pubkey) -> GenericClientResult<u64>;

    /// Get account balance, using explicit commitment
    fn get_balance_with_commitment(
        &self,
        pubkey: &Pubkey,
        commitment_config: CommitmentConfig,
    ) -> GenericClientResult<u64>;

    /// Calculate the fee for a `Message`
    fn get_fee_for_message(&self, message: &Message) -> GenericClientResult<u64>;

    /// Get the rent-exempt minimum for an account
    fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> GenericClientResult<u64>;

    /// Return the address of client
    fn addr(&self) -> String;

    /// Request, submit, and confirm an airdrop transaction
    fn request_airdrop_with_blockhash(
        &self,
        pubkey: &Pubkey,
        lamports: u64,
        recent_blockhash: &Hash,
    ) -> GenericClientResult<Signature>;

    /// Returns all information associated with the account of the provided pubkey
    fn get_account(&self, pubkey: &Pubkey) -> GenericClientResult<Account>;

    /// Returns all information associated with the account of the provided pubkey, using explicit commitment
    fn get_account_with_commitment(
        &self,
        pubkey: &Pubkey,
        commitment_config: CommitmentConfig,
    ) -> GenericClientResult<Account>;

    fn get_multiple_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> GenericClientResult<Vec<Option<Account>>>;

    fn get_slot_with_commitment(
        &self,
        commitment_config: CommitmentConfig,
    ) -> GenericClientResult<Slot>;

    fn get_blocks_with_commitment(
        &self,
        start_slot: Slot,
        end_slot: Option<Slot>,
        commitment_config: CommitmentConfig,
    ) -> GenericClientResult<Vec<Slot>>;

    fn get_block_with_config(
        &self,
        slot: Slot,
        rpc_block_config: RpcBlockConfig,
    ) -> GenericClientResult<UiConfirmedBlock>;
}

mod bank_client;
mod rpc_client;
mod tpu_client;
