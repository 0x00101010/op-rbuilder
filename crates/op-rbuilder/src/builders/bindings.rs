use alloy_sol_types::SolCall;
use base_hooks_bindings::hooks_perpetual_auction::HooksPerpetualAuction;

use crate::tx_signer::Signer;
use alloy_consensus::TxEip1559;
use alloy_primitives::{Address, Bytes, TxKind, B256, U256};
use op_alloy_consensus::OpTypedTransaction;
use reth_evm::Evm;
use reth_optimism_primitives::OpTransactionSigned;
use reth_primitives::Recovered;
use reth_provider::ProviderError;
use revm::Database;
use std::str::FromStr;

pub struct HooksPerpetualAuctionHelper;

impl HooksPerpetualAuctionHelper {
    pub fn execute_hook<E>(
        evm: &mut E,
        auction_contract: Address,
        contract_addr: Address,
        filter_hash: B256,
        topic0: B256,
        topic1: B256,
        topic2: B256,
        topic3: B256,
        event_data: Vec<u8>,
        originator: Address,
    ) -> Result<Recovered<OpTransactionSigned>, Box<dyn std::error::Error>>
    where
        E: Evm,
        E::DB: Database<Error = ProviderError>,
    {
        // Step 1: Create signer and address from private key
        let private_key_hex = "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6";
        let private_key_b256 = B256::from_str(private_key_hex)
            .map_err(|e| format!("Failed to parse private key: {:?}", e))?;
        let signer = Signer::try_from_secret(private_key_b256)
            .map_err(|e| format!("Failed to create signer: {:?}", e))?;

        let nonce = match evm.db_mut().basic(signer.address)? {
            Some(acc) => acc.nonce,
            None => {
                return Err(format!("Account not found: {:?}", signer.address).into());
            }
        };

        // Encode the function call
        let call = HooksPerpetualAuction::executeHookCall {
            contractAddr: contract_addr,
            filterHash: filter_hash,
            topic0,
            topic1,
            topic2,
            topic3,
            eventData: event_data.into(),
            originator,
        };
        let call_data = call.abi_encode();

        // Step 2: Create EIP-1559 transaction (using base_fee passed as parameter)
        let tx = OpTypedTransaction::Eip1559(TxEip1559 {
            chain_id: evm.chain_id(),
            nonce,
            gas_limit: 60_000_000,
            max_fee_per_gas: 300_000_000_000,
            max_priority_fee_per_gas: 1_000,
            to: TxKind::Call(auction_contract),
            value: U256::ZERO,
            input: Bytes::from(call_data),
            access_list: Default::default(),
        });

        // Step 3: Sign the transaction using the Signer
        let signed_tx = signer
            .sign_tx(tx)
            .map_err(|e| format!("Failed to sign transaction: {:?}", e))?;

        Ok(signed_tx)
    }
}

// Example usage:
//
// HooksPerpetualAuction Helper:
// let auction_contract = Address::from_str("0x292Fd8c1fCFE109089FB38a1528379A1Fe6Cae72").unwrap();
// let contract_addr = Address::from_str("0xd5Bf624C0c7192f13f5374070611D6f169bb5c88").unwrap();
// let topic0 = B256::from_str("0xd78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822").unwrap();
//
// Read functions:
// let hook = HooksPerpetualAuctionHelper::get_hook(&mut evm, auction_contract, contract_addr, topic0)?;
// let owner = HooksPerpetualAuctionHelper::get_owner(&mut evm, auction_contract)?;
// let gas_stipend = HooksPerpetualAuctionHelper::get_hook_gas_stipend(&mut evm, auction_contract)?;
//
// State-changing functions:
// let signer = Signer::try_from_secret(B256::from_str("0x..."))?; // Your private key
// let chain_id = 901; // OP Stack L2 chain ID
// let gas_limit = 1_000_000;
// let gas_price = 1_000_000_000; // 1 gwei
// let tx = HooksPerpetualAuctionHelper::execute_hook(
//     &mut db, auction_contract, contract_addr, topic0, topic1, topic2, topic3,
//     event_data, originator, signer, chain_id, gas_limit, gas_price
// )?;
//
