use alloy_sol_types::SolCall;
use base_hooks_bindings::{
    hooks_perpetual_auction::HooksPerpetualAuction, uniswap_v2_arb_hook::UniswapV2ArbHook,
};

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

#[allow(unused)]
impl HooksPerpetualAuctionHelper {
    pub fn get_hook(
        evm: &mut impl Evm,
        auction_contract: Address,
        contract_addr: Address,
        topic0: B256,
    ) -> Result<HooksPerpetualAuction::Hook, Box<dyn std::error::Error>> {
        let call = HooksPerpetualAuction::getHookCall {
            contractAddr: contract_addr,
            topic0,
        };

        let call_data = call.abi_encode();
        let result = evm
            .transact_system_call(Address::ZERO, auction_contract, call_data.into())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        let hook_data =
            HooksPerpetualAuction::getHookCall::abi_decode_returns(result.result.output().unwrap())
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        Ok(hook_data)
    }

    pub fn get_hooks_tuple(
        evm: &mut impl Evm,
        auction_contract: Address,
        contract_addr: Address,
        topic0: B256,
    ) -> Result<(Address, Address, U256, U256, U256), Box<dyn std::error::Error>> {
        let call = HooksPerpetualAuction::hooksCall {
            _0: contract_addr,
            _1: topic0,
        };

        let call_data = call.abi_encode();
        let result = evm
            .transact_system_call(Address::ZERO, auction_contract, call_data.into())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        let hook_data =
            HooksPerpetualAuction::hooksCall::abi_decode_returns(result.result.output().unwrap())
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        Ok((
            hook_data.owner,
            hook_data.entrypoint,
            hook_data.feePerCall,
            hook_data.deposit,
            hook_data.callsRemaining,
        ))
    }

    pub fn get_max_originator_share(
        evm: &mut impl Evm,
        auction_contract: Address,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        let call = HooksPerpetualAuction::MAX_ORIGINATOR_SHARECall {};
        let call_data = call.abi_encode();
        let result = evm
            .transact_system_call(Address::ZERO, auction_contract, call_data.into())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        let share = HooksPerpetualAuction::MAX_ORIGINATOR_SHARECall::abi_decode_returns(
            result.result.output().unwrap(),
        )
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        Ok(share)
    }

    pub fn get_min_calls_deposit(
        evm: &mut impl Evm,
        auction_contract: Address,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        let call = HooksPerpetualAuction::MIN_CALLS_DEPOSITCall {};
        let call_data = call.abi_encode();
        let result = evm
            .transact_system_call(Address::ZERO, auction_contract, call_data.into())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        let deposit = HooksPerpetualAuction::MIN_CALLS_DEPOSITCall::abi_decode_returns(
            result.result.output().unwrap(),
        )
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        Ok(deposit)
    }

    pub fn get_excess_eth(
        evm: &mut impl Evm,
        auction_contract: Address,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        let call = HooksPerpetualAuction::getExcessETHCall {};
        let call_data = call.abi_encode();
        let result = evm
            .transact_system_call(Address::ZERO, auction_contract, call_data.into())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        let excess = HooksPerpetualAuction::getExcessETHCall::abi_decode_returns(
            result.result.output().unwrap(),
        )
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        Ok(excess)
    }

    pub fn get_hook_gas_stipend(
        evm: &mut impl Evm,
        auction_contract: Address,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        let call = HooksPerpetualAuction::hookGasStipendCall {};
        let call_data = call.abi_encode();
        let result = evm
            .transact_system_call(Address::ZERO, auction_contract, call_data.into())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        let stipend = HooksPerpetualAuction::hookGasStipendCall::abi_decode_returns(
            result.result.output().unwrap(),
        )
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        Ok(stipend)
    }

    pub fn get_originator_share_bps(
        evm: &mut impl Evm,
        auction_contract: Address,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        let call = HooksPerpetualAuction::originatorShareBpsCall {};
        let call_data = call.abi_encode();
        let result = evm
            .transact_system_call(Address::ZERO, auction_contract, call_data.into())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        let share_bps = HooksPerpetualAuction::originatorShareBpsCall::abi_decode_returns(
            result.result.output().unwrap(),
        )
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        Ok(share_bps)
    }

    pub fn get_owner(
        evm: &mut impl Evm,
        auction_contract: Address,
    ) -> Result<Address, Box<dyn std::error::Error>> {
        let call = HooksPerpetualAuction::ownerCall {};
        let call_data = call.abi_encode();
        let result = evm
            .transact_system_call(Address::ZERO, auction_contract, call_data.into())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        let owner =
            HooksPerpetualAuction::ownerCall::abi_decode_returns(result.result.output().unwrap())
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        Ok(owner)
    }

    pub fn get_total_reserved_eth(
        evm: &mut impl Evm,
        auction_contract: Address,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        let call = HooksPerpetualAuction::totalReservedETHCall {};
        let call_data = call.abi_encode();
        let result = evm
            .transact_system_call(Address::ZERO, auction_contract, call_data.into())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        let reserved = HooksPerpetualAuction::totalReservedETHCall::abi_decode_returns(
            result.result.output().unwrap(),
        )
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        Ok(reserved)
    }

    pub fn execute_hook<E>(
        evm: &mut E,
        auction_contract: Address,
        contract_addr: Address,
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

    /// Get multiple hooks for different contract/topic pairs
    pub fn get_all_hooks(
        evm: &mut impl Evm,
        auction_contract: Address,
        hook_pairs: Vec<(Address, B256)>,
    ) -> Result<Vec<(Address, B256, HooksPerpetualAuction::Hook)>, Box<dyn std::error::Error>> {
        let mut hooks = Vec::new();

        for (contract_addr, topic0) in hook_pairs {
            match Self::get_hook(evm, auction_contract, contract_addr, topic0) {
                Ok(hook) => {
                    hooks.push((contract_addr, topic0, hook));
                }
                Err(e) => {
                    // Log error but continue with other hooks
                    eprintln!(
                        "Failed to get hook for {:?}/{:?}: {}",
                        contract_addr, topic0, e
                    );
                }
            }
        }

        Ok(hooks)
    }

    /// Get all hooks for a specific contract across multiple topics
    pub fn get_hooks_for_contract(
        evm: &mut impl Evm,
        auction_contract: Address,
        contract_addr: Address,
        topics: Vec<B256>,
    ) -> Result<Vec<(B256, HooksPerpetualAuction::Hook)>, Box<dyn std::error::Error>> {
        let mut hooks = Vec::new();

        for topic0 in topics {
            match Self::get_hook(evm, auction_contract, contract_addr, topic0) {
                Ok(hook) => {
                    // Only include hooks that have a valid owner (not zero address)
                    if hook.owner != Address::ZERO {
                        hooks.push((topic0, hook));
                    }
                }
                Err(e) => {
                    // Log error but continue with other topics
                    eprintln!(
                        "Failed to get hook for {:?}/{:?}: {}",
                        contract_addr, topic0, e
                    );
                }
            }
        }

        Ok(hooks)
    }

    /// Get all hooks for a list of contracts and a specific topic
    pub fn get_hooks_for_topic(
        evm: &mut impl Evm,
        auction_contract: Address,
        topic0: B256,
        contracts: Vec<Address>,
    ) -> Result<Vec<(Address, HooksPerpetualAuction::Hook)>, Box<dyn std::error::Error>> {
        let mut hooks = Vec::new();

        for contract_addr in contracts {
            match Self::get_hook(evm, auction_contract, contract_addr, topic0) {
                Ok(hook) => {
                    // Only include hooks that have a valid owner (not zero address)
                    if hook.owner != Address::ZERO {
                        hooks.push((contract_addr, hook));
                    }
                }
                Err(e) => {
                    // Log error but continue with other contracts
                    eprintln!(
                        "Failed to get hook for {:?}/{:?}: {}",
                        contract_addr, topic0, e
                    );
                }
            }
        }

        Ok(hooks)
    }

    /// Check if a hook exists for a specific contract/topic pair
    pub fn hook_exists(
        evm: &mut impl Evm,
        auction_contract: Address,
        contract_addr: Address,
        topic0: B256,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        match Self::get_hook(evm, auction_contract, contract_addr, topic0) {
            Ok(hook) => Ok(hook.owner != Address::ZERO && hook.callsRemaining > U256::ZERO),
            Err(_) => Ok(false),
        }
    }

    /// Get hook summary information (useful for monitoring/debugging)
    pub fn get_hook_summary(
        evm: &mut impl Evm,
        auction_contract: Address,
        contract_addr: Address,
        topic0: B256,
    ) -> Result<Option<(Address, Address, u64, u64, u64)>, Box<dyn std::error::Error>> {
        match Self::get_hook(evm, auction_contract, contract_addr, topic0) {
            Ok(hook) => {
                if hook.owner != Address::ZERO {
                    Ok(Some((
                        hook.owner,
                        hook.entrypoint,
                        hook.feePerCall.to::<u64>(),
                        hook.deposit.to::<u64>(),
                        hook.callsRemaining.to::<u64>(),
                    )))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }

    /// Try to discover hooks by scanning for common contract addresses and event signatures.
    /// Note: This is a best-effort approach since smart contract mappings cannot be enumerated directly.
    /// You must provide the contract addresses and topics you want to scan.
    pub fn scan_for_hooks(
        evm: &mut impl Evm,
        auction_contract: Address,
        candidate_contracts: Vec<Address>,
        candidate_topics: Vec<B256>,
    ) -> Result<Vec<(Address, B256, HooksPerpetualAuction::Hook)>, Box<dyn std::error::Error>> {
        let mut found_hooks = Vec::new();

        for contract_addr in candidate_contracts {
            for topic0 in &candidate_topics {
                if let Ok(true) = Self::hook_exists(evm, auction_contract, contract_addr, *topic0) {
                    if let Ok(hook) = Self::get_hook(evm, auction_contract, contract_addr, *topic0)
                    {
                        found_hooks.push((contract_addr, *topic0, hook));
                    }
                }
            }
        }

        Ok(found_hooks)
    }

    /// Place a bid on the HooksPerpetualAuction contract
    pub fn place_bid<E>(
        evm: &mut E,
        auction_contract: Address,
        contract_addr: Address,
        topic0: B256,
        entrypoint: Address,
        fee_per_call: U256,
        calls_to_deposit: U256,
    ) -> Result<Recovered<OpTransactionSigned>, Box<dyn std::error::Error>>
    where
        E: Evm,
        E::DB: Database<Error = ProviderError>,
    {
        // Create signer from private key
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

        // Encode the bid function call
        let call = HooksPerpetualAuction::bidCall {
            contractAddr: contract_addr,
            topic0,
            entrypoint,
            feePerCall: fee_per_call,
            callsToDeposit: calls_to_deposit,
        };
        let call_data = call.abi_encode();

        // Calculate total value to send (fee_per_call * calls_to_deposit)
        let total_value = fee_per_call * calls_to_deposit;

        // Create EIP-1559 transaction
        let tx = OpTypedTransaction::Eip1559(TxEip1559 {
            chain_id: evm.chain_id(),
            nonce,
            gas_limit: 1_000_000,
            max_fee_per_gas: 1_000_000_000, // 1 gwei
            max_priority_fee_per_gas: 1_000,
            to: TxKind::Call(auction_contract),
            value: total_value,
            input: Bytes::from(call_data),
            access_list: Default::default(),
        });

        // Sign the transaction
        let signed_tx = signer
            .sign_tx(tx)
            .map_err(|e| format!("Failed to sign transaction: {:?}", e))?;

        Ok(signed_tx)
    }

    /// Convenience function to place a bid for Uniswap V2 Swap events
    pub fn place_swap_bid<E>(
        evm: &mut E,
        auction_contract: Address,
        pair_address: Address,
        arb_hook_address: Address,
        fee_per_call_eth: u64, // Fee per call in ETH (e.g., 1 for 1 ETH)
        calls_to_deposit: u64, // Number of calls to deposit (e.g., 100)
    ) -> Result<Recovered<OpTransactionSigned>, Box<dyn std::error::Error>>
    where
        E: Evm,
        E::DB: Database<Error = ProviderError>,
    {
        // Uniswap V2 Swap event topic hash
        let swap_topic0 =
            B256::from_str("0xd78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822")
                .map_err(|e| format!("Invalid topic hash: {:?}", e))?;

        // Convert ETH to wei
        let fee_per_call = U256::from(fee_per_call_eth) * U256::from(10u64.pow(18));
        let calls = U256::from(calls_to_deposit);

        Self::place_bid(
            evm,
            auction_contract,
            pair_address,
            swap_topic0,
            arb_hook_address,
            fee_per_call,
            calls,
        )
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
