use {
    crate::{GenericClient, GenericClientError, GenericClientResult},
    solana_connection_cache::connection_cache::{
        ConnectionManager, ConnectionPool, NewConnectionConfig,
    },
    solana_rpc_client_api::config::RpcBlockConfig,
    solana_sdk::{
        account::Account, commitment_config::CommitmentConfig, epoch_info::EpochInfo, hash::Hash,
        message::Message, pubkey::Pubkey, signature::Signature, slot_history::Slot,
        transaction::Transaction,
    },
    solana_tpu_client::tpu_client::TpuClient,
    solana_transaction_status::UiConfirmedBlock,
};

impl<P, M, C> GenericClient for TpuClient<P, M, C>
where
    P: ConnectionPool<NewConnectionConfig = C>,
    M: ConnectionManager<ConnectionPool = P, NewConnectionConfig = C>,
    C: NewConnectionConfig,
{
    fn send_transaction(&self, transaction: Transaction) -> GenericClientResult<Signature> {
        let signature = transaction.signatures[0];
        self.try_send_transaction(&transaction)?;
        Ok(signature)
    }
    fn send_batch(&self, transactions: Vec<Transaction>) -> GenericClientResult<()> {
        self.try_send_transaction_batch(&transactions)?;
        Ok(())
    }
    fn get_latest_blockhash(&self) -> GenericClientResult<Hash> {
        self.rpc_client()
            .get_latest_blockhash()
            .map_err(|err| err.into())
    }

    fn get_latest_blockhash_with_commitment(
        &self,
        commitment_config: CommitmentConfig,
    ) -> GenericClientResult<(Hash, u64)> {
        self.rpc_client()
            .get_latest_blockhash_with_commitment(commitment_config)
            .map_err(|err| err.into())
    }

    fn get_transaction_count(&self) -> GenericClientResult<u64> {
        self.rpc_client()
            .get_transaction_count()
            .map_err(|err| err.into())
    }

    fn get_transaction_count_with_commitment(
        &self,
        commitment_config: CommitmentConfig,
    ) -> GenericClientResult<u64> {
        self.rpc_client()
            .get_transaction_count_with_commitment(commitment_config)
            .map_err(|err| err.into())
    }

    fn get_epoch_info(&self) -> GenericClientResult<EpochInfo> {
        self.rpc_client().get_epoch_info().map_err(|err| err.into())
    }

    fn get_balance(&self, pubkey: &Pubkey) -> GenericClientResult<u64> {
        self.rpc_client()
            .get_balance(pubkey)
            .map_err(|err| err.into())
    }

    fn get_balance_with_commitment(
        &self,
        pubkey: &Pubkey,
        commitment_config: CommitmentConfig,
    ) -> GenericClientResult<u64> {
        self.rpc_client()
            .get_balance_with_commitment(pubkey, commitment_config)
            .map(|res| res.value)
            .map_err(|err| err.into())
    }

    fn get_fee_for_message(&self, message: &Message) -> GenericClientResult<u64> {
        self.rpc_client()
            .get_fee_for_message(message)
            .map_err(|err| err.into())
    }

    fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> GenericClientResult<u64> {
        self.rpc_client()
            .get_minimum_balance_for_rent_exemption(data_len)
            .map_err(|err| err.into())
    }

    fn addr(&self) -> String {
        self.rpc_client().url()
    }

    fn request_airdrop_with_blockhash(
        &self,
        pubkey: &Pubkey,
        lamports: u64,
        recent_blockhash: &Hash,
    ) -> GenericClientResult<Signature> {
        self.rpc_client()
            .request_airdrop_with_blockhash(pubkey, lamports, recent_blockhash)
            .map_err(|err| err.into())
    }

    fn get_account(&self, pubkey: &Pubkey) -> GenericClientResult<Account> {
        self.rpc_client()
            .get_account(pubkey)
            .map_err(|err| err.into())
    }

    fn get_account_with_commitment(
        &self,
        pubkey: &Pubkey,
        commitment_config: CommitmentConfig,
    ) -> GenericClientResult<Account> {
        self.rpc_client()
            .get_account_with_commitment(pubkey, commitment_config)
            .map(|res| res.value)
            .map_err(|err| err.into())
            .and_then(|account| {
                account.ok_or_else(|| {
                    GenericClientError::Custom(format!("AccountNotFound: pubkey={pubkey}"))
                })
            })
    }

    fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> GenericClientResult<Vec<Option<Account>>> {
        self.rpc_client()
            .get_multiple_accounts(pubkeys)
            .map_err(|err| err.into())
    }

    fn get_slot_with_commitment(&self, commitment_config: CommitmentConfig) -> GenericClientResult<Slot> {
        self.rpc_client()
            .get_slot_with_commitment(commitment_config)
            .map_err(|err| err.into())
    }

    fn get_blocks_with_commitment(
        &self,
        start_slot: Slot,
        end_slot: Option<Slot>,
        commitment_config: CommitmentConfig,
    ) -> GenericClientResult<Vec<Slot>> {
        self.rpc_client()
            .get_blocks_with_commitment(start_slot, end_slot, commitment_config)
            .map_err(|err| err.into())
    }

    fn get_block_with_config(
        &self,
        slot: Slot,
        rpc_block_config: RpcBlockConfig,
    ) -> GenericClientResult<UiConfirmedBlock> {
        self.rpc_client()
            .get_block_with_config(slot, rpc_block_config)
            .map_err(|err| err.into())
    }
}
