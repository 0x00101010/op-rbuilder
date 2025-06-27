use alloy_primitives::{Address, Log, B256};
use alloy_sol_types::SolEvent;
use base_hooks_bindings::hooks_perpetual_auction::HooksPerpetualAuction::{Hook, NewBid};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio::sync::mpsc;
use tracing::{error, info};

/// Maintains a mapping of contract addresses and topics to their hook data
#[derive(Debug)]
pub struct HooksIndexer {
    /// Mapping from (contract_address, topic0) to Hook data
    /// Using RwLock for frequent reads with occasional writes
    hook_registry: Arc<RwLock<HashMap<(Address, B256), Hook>>>,
    /// Channel sender for outgoing hook events
    event_sender: mpsc::UnboundedSender<Log>,
}

impl HooksIndexer {
    /// Creates a new hooks indexer and spawns the processing task
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let hook_registry = Arc::new(RwLock::new(HashMap::new()));

        // Spawn the background processing task
        tokio::spawn(processing_loop(event_receiver, hook_registry.clone()));

        Self {
            hook_registry,
            event_sender,
        }
    }

    /// Gets hook data for a specific contract and topic (non-async)
    /// Returns a cloned Hook to avoid holding the read lock
    pub fn get_hook(&self, contract_address: Address, topic0: B256) -> Option<Hook> {
        let key = (contract_address, topic0);
        self.hook_registry.read().ok()?.get(&key).cloned()
    }

    /// Sends a log to be processed by the background task
    pub fn send_log(&self, log: Log) {
        if let Err(e) = self.event_sender.send(log) {
            error!("Failed to send log to hooks indexer: {:?}", e);
        }
    }
}

/// Background processing loop that updates the hook registry based on incoming logs
async fn processing_loop(
    mut receiver: mpsc::UnboundedReceiver<Log>,
    hook_registry: Arc<RwLock<HashMap<(Address, B256), Hook>>>,
) {
    while let Some(log) = receiver.recv().await {
        // Check if this is a NewBid event
        if log.topics()[0] == NewBid::SIGNATURE_HASH {
            match NewBid::decode_log_validate(&log) {
                Ok(new_bid) => {
                    let key = (new_bid.contractAddr, new_bid.topic0);
                    let hook = Hook {
                        owner: new_bid.bidder,
                        entrypoint: new_bid.entrypoint,
                        feePerCall: new_bid.feePerCall,
                        deposit: new_bid.feePerCall * new_bid.callsDeposited,
                        callsRemaining: new_bid.callsDeposited,
                    };

                    info!(
                        "Processing NewBid event: contract={:?}, topic={:?}, bidder={:?}",
                        new_bid.contractAddr, new_bid.topic0, new_bid.bidder
                    );

                    // Update the registry
                    if let Ok(mut registry) = hook_registry.write() {
                        registry.insert(key, hook);
                        info!("Updated hook registry, total hooks: {}", registry.len());
                    } else {
                        error!("Failed to acquire write lock for hook registry");
                    }
                }
                Err(e) => {
                    error!("Failed to decode NewBid log: {:?}", e);
                }
            }
        }
    }
}
