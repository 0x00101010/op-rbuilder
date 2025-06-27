use alloy_primitives::{Address, FixedBytes, Log, B256, U256};
use alloy_sol_types::SolEvent;
use base_hooks_bindings::hooks_perpetual_auction::HooksPerpetualAuction::{
    EventFilter, Hook, HookExecuted, NewBid, TopicFilter,
};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, RwLock},
};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Maintains a mapping of contract addresses and topics to their hook data
#[derive(Debug)]
pub struct HooksIndexer {
    /// Mapping from (contract_address, topic0) to Hook data
    /// Using RwLock for frequent reads with occasional writes
    hook_registry: Arc<RwLock<HashMap<(Address, B256), Vec<HookData>>>>,
    /// Channel sender for outgoing hook events
    event_sender: mpsc::UnboundedSender<Log>,
}

#[derive(Clone, Debug)]
pub struct HookData {
    pub _inner: Hook,
    pub filter_hash: FixedBytes<32>,
}

impl HooksIndexer {
    /// Creates a new hooks indexer and spawns the Øprocessing task
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

    pub fn get_hook_to_execute(&self, log: &Log) -> Option<HookData> {
        // Get hooks matching contract + topic0
        let hooks = self.get_minimum_matching_hooks(&log);

        // 2. Filter hooks that match on event filters
        let matching_hooks: Vec<_> = hooks
            .into_iter()
            .filter(|hook| self.matches_event_filter(&log, &hook._inner.filter))
            .collect();

        // 3. Group by feePerCall and sort by specificity within groups
        let mut fee_groups: BTreeMap<U256, Vec<_>> = BTreeMap::new();
        for hook in matching_hooks {
            let specificity = self.calculate_specificity(&hook._inner.filter);
            fee_groups
                .entry(hook._inner.feePerCall)
                .or_default()
                .push((hook, specificity));
        }

        // 4. Sort each group by specificity (least specific first)
        for (_, hooks) in fee_groups.iter_mut() {
            hooks.sort_by_key(|(_, specificity)| *specificity);
        }

        // 5. Return hook from highest fee tier with least specificity
        fee_groups
            .into_iter()
            .rev() // Highest fees first
            .next()
            .and_then(|(_, hooks)| hooks.into_iter().next())
            .map(|(hook, _)| hook)
    }

    fn get_minimum_matching_hooks(&self, log: &Log) -> Vec<HookData> {
        let topic0 = log.topics()[0];
        let key = (log.address, topic0);
        let hooks = self.hook_registry.read().unwrap().get(&key).cloned();
        match hooks {
            Some(hooks) => hooks,
            None => vec![],
        }
    }

    fn matches_event_filter(&self, log: &Log, filter: &EventFilter) -> bool {
        let topics = log.topics();

        if filter.topic1.enabled {
            info!("Matching topic1: {:?}", filter.topic1);
            let topic1 = topics.get(1).copied();
            if topic1.is_none() {
                return false;
            }
            if !self.matches_topic_filter(&topic1.unwrap(), &filter.topic1) {
                return false;
            }
        }

        if filter.topic2.enabled {
            let topic2 = topics.get(2).copied();
            if topic2.is_none() {
                return false;
            }
            if !self.matches_topic_filter(&topic2.unwrap(), &filter.topic2) {
                return false;
            }
        }

        if filter.topic3.enabled {
            let topic3 = topics.get(3).copied();
            if topic3.is_none() {
                return false;
            }
            if !self.matches_topic_filter(&topic3.unwrap(), &filter.topic3) {
                return false;
            }
        }

        true
    }

    fn matches_topic_filter(&self, topic: &FixedBytes<32>, filter: &TopicFilter) -> bool {
        let topic_u256 = U256::from_be_bytes(topic.0);
        let filter_u256 = U256::from_be_bytes(filter.value.0);

        info!(
            "Matching topic: {:?}, filter: {:?}",
            topic_u256, filter_u256
        );

        match filter.op {
            // NONE
            0 => {
                return true;
            }
            // EQ
            1 => {
                return topic_u256 == filter_u256;
            }
            // LT
            2 => {
                return topic_u256 < filter_u256;
            }
            // LTE
            3 => {
                return topic_u256 <= filter_u256;
            }
            // GT
            4 => {
                return topic_u256 > filter_u256;
            }
            // GTE
            5 => {
                return topic_u256 >= filter_u256;
            }
            _ => {
                error!("Invalid topic filter operation: {:?}", filter.op);
                return false;
            }
        }
    }

    fn calculate_specificity(&self, filter: &EventFilter) -> u8 {
        filter.topic1.enabled as u8 + filter.topic2.enabled as u8 + filter.topic3.enabled as u8
    }

    /// Sends a log to be processed by the background task
    pub fn send_log(&self, log: &Log) {
        info!("Sending log to hooks indexer: {:?}", log);
        if let Err(e) = self.event_sender.send(log.clone()) {
            error!("Failed to send log to hooks indexer: {:?}", e);
        }
    }
}

/// Background processing loop that updates the hook registry based on incoming logs
async fn processing_loop(
    mut receiver: mpsc::UnboundedReceiver<Log>,
    hook_registry: Arc<RwLock<HashMap<(Address, B256), Vec<HookData>>>>,
) {
    while let Some(log) = receiver.recv().await {
        let topic0 = log.topics()[0];

        match topic0 {
            NewBid::SIGNATURE_HASH => {
                match NewBid::decode_log_validate(&log) {
                    Ok(new_bid) => {
                        let key = (new_bid.contractAddr, new_bid.topic0);
                        let hook_data = HookData {
                            _inner: Hook {
                                topic0: new_bid.topic0,
                                owner: new_bid.bidder,
                                entrypoint: new_bid.entrypoint,
                                feePerCall: new_bid.feePerCall,
                                deposit: new_bid.feePerCall * new_bid.callsDeposited,
                                callsRemaining: new_bid.callsDeposited,
                                filter: new_bid.filter.clone(),
                            },
                            filter_hash: new_bid.filterHash,
                        };

                        info!(
                            "Processing NewBid event: contract={:?}, topic={:?}, bidder={:?}",
                            new_bid.contractAddr, new_bid.topic0, new_bid.bidder
                        );

                        // Update the registry
                        if let Ok(mut registry) = hook_registry.write() {
                            registry
                                .entry(key)
                                .and_modify(|hooks| hooks.push(hook_data.clone()))
                                .or_insert(vec![hook_data]);
                            info!(
                                "Updated hook registry, total hooks: {:?}",
                                registry.entry(key).or_default().len()
                            );
                        } else {
                            error!("Failed to acquire write lock for hook registry");
                        }
                    }
                    Err(e) => {
                        error!("Failed to decode NewBid log: {:?}", e);
                    }
                }
            }
            HookExecuted::SIGNATURE_HASH => match HookExecuted::decode_log_validate(&log) {
                Ok(hook_executed) => {
                    let key = (hook_executed.contractAddr, hook_executed.topic0);
                    let existing_hooks = hook_registry
                        .read()
                        .unwrap()
                        .get(&key)
                        .cloned()
                        .expect("Hook topic0 should have existed if it was executed");
                    let exact_hook = existing_hooks
                        .iter()
                        .find(|hook| hook.filter_hash == hook_executed.filterHash)
                        .expect("Hook filterHash should have existed if it was executed");

                    let new_hook_data = HookData {
                        _inner: Hook {
                            deposit: exact_hook._inner.deposit - hook_executed.feePerCall,
                            callsRemaining: exact_hook
                                ._inner
                                .callsRemaining
                                .saturating_sub(U256::from(1)),
                            filter: exact_hook._inner.filter.clone(),
                            ..exact_hook._inner
                        },
                        filter_hash: exact_hook.filter_hash,
                    };
                    if new_hook_data._inner.callsRemaining == U256::ZERO {
                        // Remove the hook from the registry
                        if let Ok(mut registry) = hook_registry.write() {
                            registry.entry(key).and_modify(|hooks| {
                                hooks.retain(|hook| hook.filter_hash != exact_hook.filter_hash)
                            });
                            info!(
                                "Removed hook from registry, total hooks: {}",
                                registry.len()
                            );
                        } else {
                            error!("Failed to acquire write lock for hook registry");
                        }
                    } else {
                        // Update the registry
                        if let Ok(mut registry) = hook_registry.write() {
                            registry
                                .entry(key)
                                .and_modify(|hooks| {
                                    if let Some(hook_index) = hooks
                                        .iter()
                                        .position(|hook| hook.filter_hash == exact_hook.filter_hash)
                                    {
                                        hooks[hook_index] = new_hook_data;
                                    }
                                })
                                .or_insert(Vec::new());
                            info!("Updated hook registry, total hooks: {}", registry.len());
                        } else {
                            error!("Failed to acquire write lock for hook registry");
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to decode HookExecuted log: {:?}", e);
                }
            },
            _ => {
                warn!(
                    "HooksIndexer: received log with unhandled topic: {:?}",
                    topic0
                );
            }
        }
    }
}
