use {
    crate::{GenericClient, GenericClientError, GenericClientResult},
    solana_rpc_client::rpc_client::RpcClient,
    solana_rpc_client_api::config::RpcBlockConfig,
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
    },
    solana_transaction_status::UiConfirmedBlock,
};

impl GenericClient for RpcClient {
    fn send_transaction(&self, transaction: Transaction) -> GenericClientResult<Signature> {
        RpcClient::send_transaction(self, &transaction).map_err(|err| err.into())
    }

    fn send_batch(&self, transactions: Vec<Transaction>) -> GenericClientResult<()> {
        for transaction in transactions {
            GenericClient::send_transaction(self, transaction)?;
        }
        Ok(())
    }
    fn get_latest_blockhash(&self) -> GenericClientResult<Hash> {
        RpcClient::get_latest_blockhash(self).map_err(|err| err.into())
    }

    fn get_latest_blockhash_with_commitment(
        &self,
        commitment_config: CommitmentConfig,
    ) -> GenericClientResult<(Hash, u64)> {
        RpcClient::get_latest_blockhash_with_commitment(self, commitment_config)
            .map_err(|err| err.into())
    }

    fn get_new_latest_blockhash(&self, blockhash: &Hash) -> GenericClientResult<Hash> {
        RpcClient::get_new_latest_blockhash(self, blockhash).map_err(|err| err.into())
    }

    fn get_signature_status(
        &self,
        signature: &Signature,
    ) -> GenericClientResult<Option<Result<()>>> {
        RpcClient::get_signature_status(self, signature).map_err(|err| err.into())
    }

    fn get_transaction_count(&self) -> GenericClientResult<u64> {
        RpcClient::get_transaction_count(self).map_err(|err| err.into())
    }

    fn get_transaction_count_with_commitment(
        &self,
        commitment_config: CommitmentConfig,
    ) -> GenericClientResult<u64> {
        RpcClient::get_transaction_count_with_commitment(self, commitment_config)
            .map_err(|err| err.into())
    }

    fn get_epoch_info(&self) -> GenericClientResult<EpochInfo> {
        RpcClient::get_epoch_info(self).map_err(|err| err.into())
    }

    fn get_balance(&self, pubkey: &Pubkey) -> GenericClientResult<u64> {
        RpcClient::get_balance(self, pubkey).map_err(|err| err.into())
    }

    fn get_balance_with_commitment(
        &self,
        pubkey: &Pubkey,
        commitment_config: CommitmentConfig,
    ) -> GenericClientResult<u64> {
        RpcClient::get_balance_with_commitment(self, pubkey, commitment_config)
            .map(|res| res.value)
            .map_err(|err| err.into())
    }

    fn get_fee_for_message(&self, message: &Message) -> GenericClientResult<u64> {
        RpcClient::get_fee_for_message(self, message).map_err(|err| err.into())
    }

    fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> GenericClientResult<u64> {
        RpcClient::get_minimum_balance_for_rent_exemption(self, data_len).map_err(|err| err.into())
    }

    fn addr(&self) -> String {
        self.url()
    }

    fn request_airdrop_with_blockhash(
        &self,
        pubkey: &Pubkey,
        lamports: u64,
        recent_blockhash: &Hash,
    ) -> GenericClientResult<Signature> {
        RpcClient::request_airdrop_with_blockhash(self, pubkey, lamports, recent_blockhash)
            .map_err(|err| err.into())
    }

    fn get_account(&self, pubkey: &Pubkey) -> GenericClientResult<Account> {
        RpcClient::get_account(self, pubkey).map_err(|err| err.into())
    }

    fn get_account_with_commitment(
        &self,
        pubkey: &Pubkey,
        commitment_config: CommitmentConfig,
    ) -> GenericClientResult<Account> {
        RpcClient::get_account_with_commitment(self, pubkey, commitment_config)
            .map(|res| res.value)
            .map_err(|err| err.into())
            .and_then(|account| {
                account.ok_or_else(|| {
                    GenericClientError::Custom(format!("AccountNotFound: pubkey={pubkey}"))
                })
            })
    }

    fn get_multiple_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> GenericClientResult<Vec<Option<Account>>> {
        RpcClient::get_multiple_accounts(self, pubkeys).map_err(|err| err.into())
    }

    fn get_slot_with_commitment(
        &self,
        commitment_config: CommitmentConfig,
    ) -> GenericClientResult<Slot> {
        RpcClient::get_slot_with_commitment(self, commitment_config).map_err(|err| err.into())
    }

    fn get_blocks_with_commitment(
        &self,
        start_slot: Slot,
        end_slot: Option<Slot>,
        commitment_config: CommitmentConfig,
    ) -> GenericClientResult<Vec<Slot>> {
        RpcClient::get_blocks_with_commitment(self, start_slot, end_slot, commitment_config)
            .map_err(|err| err.into())
    }

    fn get_block_with_config(
        &self,
        slot: Slot,
        rpc_block_config: RpcBlockConfig,
    ) -> GenericClientResult<UiConfirmedBlock> {
        RpcClient::get_block_with_config(self, slot, rpc_block_config).map_err(|err| err.into())
    }
}
