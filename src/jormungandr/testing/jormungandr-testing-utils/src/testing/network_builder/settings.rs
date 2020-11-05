use super::LegacyWalletTemplate;
use crate::testing::network_builder::{
    Blockchain as BlockchainTemplate, Node as NodeTemplate, NodeAlias, Random, Wallet, WalletAlias,
    WalletTemplate, WalletType,
};
use crate::{stake_pool::StakePool, testing::signed_stake_pool_cert, wallet::Wallet as WalletLib};
use chain_crypto::Ed25519;
use chain_impl_mockchain::{
    certificate::VotePlan,
    chaintypes::ConsensusVersion,
    fee::LinearFee,
    testing::{create_initial_vote_plan, scenario::template::VotePlanDef},
};
use jormungandr_lib::{
    crypto::key::SigningKey,
    interfaces::{
        try_initials_vec_from_messages, ActiveSlotCoefficient, Bft, Block0Configuration,
        BlockchainConfiguration, CommitteeIdDef, GenesisPraos, Initial, InitialUTxO, LegacyUTxO,
        NodeConfig, NodeSecret,
    },
};
use rand_core::{CryptoRng, RngCore};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use vit_servicing_station_lib::server::settings::ServiceSettings;

pub type VotePlanAlias = String;

/// contains all the data to start or interact with a node
#[derive(Debug, Clone)]
pub struct NodeSetting {
    /// for reference purpose only
    pub alias: NodeAlias,

    /// node secret, this will be passed to the node at start
    /// up of the node. It may contains the necessary crypto
    /// for the node to be a blockchain leader (BFT leader or
    /// stake pool)
    pub secret: NodeSecret,

    pub config: NodeConfig,

    pub node_topology: NodeTemplate,
}

impl NodeSetting {
    pub fn new(
        alias: NodeAlias,
        config: NodeConfig,
        secret: NodeSecret,
        template: NodeTemplate,
    ) -> Self {
        Self {
            alias,
            config,
            secret,
            node_topology: template,
        }
    }

    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    pub fn secrets(&self) -> &NodeSecret {
        &self.secret
    }
}

#[derive(Clone, Debug)]
pub struct WalletProxySettings {
    pub proxy_address: SocketAddr,
    pub vit_station_address: SocketAddr,
    pub node_backend_address: Option<SocketAddr>,
}

impl WalletProxySettings {
    pub fn base_address(&self) -> SocketAddr {
        self.proxy_address.clone()
    }

    pub fn base_vit_address(&self) -> SocketAddr {
        self.vit_station_address.clone()
    }

    pub fn base_node_backend_address(&self) -> Option<SocketAddr> {
        self.node_backend_address.clone()
    }

    pub fn address(&self) -> String {
        format!("http://{}", self.base_address())
    }

    pub fn vit_address(&self) -> String {
        format!("http://{}", self.base_vit_address())
    }

    pub fn node_backend_address(&self) -> String {
        format!(
            "http://{}/api/v0",
            self.base_node_backend_address().unwrap()
        )
    }
}

#[derive(Debug)]
pub struct Settings {
    pub nodes: HashMap<NodeAlias, NodeSetting>,

    pub vit_stations: HashMap<NodeAlias, ServiceSettings>,

    pub wallet_proxies: HashMap<NodeAlias, WalletProxySettings>,

    pub wallets: HashMap<WalletAlias, Wallet>,

    pub legacy_wallets: HashMap<WalletAlias, LegacyWalletTemplate>,

    pub block0: Block0Configuration,

    pub stake_pools: HashMap<NodeAlias, StakePool>,

    pub vote_plans: HashMap<VotePlanAlias, VotePlan>,
}

impl Settings {
    pub fn new<RNG>(
        nodes: HashMap<NodeAlias, NodeSetting>,
        blockchain: BlockchainTemplate,
        vit_stations: HashMap<NodeAlias, ServiceSettings>,
        wallet_proxies: HashMap<NodeAlias, WalletProxySettings>,
        rng: &mut Random<RNG>,
    ) -> Self
    where
        RNG: RngCore + CryptoRng,
    {
        let mut settings = Settings {
            nodes,
            wallets: HashMap::new(),
            block0: Block0Configuration {
                blockchain_configuration: BlockchainConfiguration::new(
                    chain_addr::Discrimination::Test,
                    ConsensusVersion::Bft,
                    LinearFee::new(1, 2, 3),
                ),
                initial: Vec::new(),
            },
            vit_stations,
            wallet_proxies: wallet_proxies,
            legacy_wallets: HashMap::new(),
            stake_pools: HashMap::new(),
            vote_plans: HashMap::new(),
        };

        settings.populate_trusted_peers();
        settings.populate_block0_blockchain_initials(blockchain.wallets(), rng);
        settings.populate_block0_blockchain_configuration(&blockchain, rng);
        settings.populate_block0_blockchain_legacy(blockchain.legacy_wallets(), rng);
        settings.populate_block0_blockchain_vote_plans(blockchain.vote_plans());

        println!("{:?}", settings);

        settings
    }

    fn populate_block0_blockchain_legacy<RNG>(
        &mut self,
        legacy_wallets: Vec<LegacyWalletTemplate>,
        _rng: &mut Random<RNG>,
    ) where
        RNG: RngCore + CryptoRng,
    {
        for template in legacy_wallets {
            let legacy_fragment = Initial::LegacyFund(vec![LegacyUTxO {
                address: template.address().parse().unwrap(),
                value: *template.value(),
            }]);

            self.legacy_wallets
                .insert(template.alias().clone(), template.clone());
            self.block0.initial.push(legacy_fragment);
        }
    }

    fn populate_block0_blockchain_vote_plans(&mut self, vote_plans: Vec<VotePlanDef>) {
        let mut vote_plans_fragments = Vec::new();
        for vote_plan_def in vote_plans {
            let owner = self.wallets.get(&vote_plan_def.owner()).expect(&format!(
                "Owner {} of {} is unknown wallet ",
                vote_plan_def.owner(),
                vote_plan_def.alias()
            ));
            let vote_plan: VotePlan = vote_plan_def.into();
            vote_plans_fragments.push(create_initial_vote_plan(
                &vote_plan,
                &[owner.clone().into()],
            ));
        }
        self.block0
            .initial
            .extend(try_initials_vec_from_messages(vote_plans_fragments.iter()).unwrap())
    }

    fn populate_block0_blockchain_configuration<RNG>(
        &mut self,
        blockchain: &BlockchainTemplate,
        rng: &mut Random<RNG>,
    ) where
        RNG: RngCore + CryptoRng,
    {
        let mut blockchain_configuration = &mut self.block0.blockchain_configuration;

        // TODO blockchain_configuration.block0_date = ;
        blockchain_configuration.discrimination = chain_addr::Discrimination::Test;
        blockchain_configuration.block0_consensus = *blockchain.consensus();
        blockchain_configuration.consensus_leader_ids = {
            let mut leader_ids = Vec::new();
            for leader_alias in blockchain.leaders() {
                let identifier = if let Some(node) = self.nodes.get_mut(leader_alias) {
                    if let Some(bft) = &node.secret.bft {
                        bft.signing_key.identifier()
                    } else {
                        let signing_key = SigningKey::generate(rng.rng_mut());
                        let identifier = signing_key.identifier();
                        node.secret.bft = Some(Bft { signing_key });
                        identifier
                    }
                } else {
                    SigningKey::<Ed25519>::generate(rng.rng_mut()).identifier()
                };
                leader_ids.push(identifier.into());
            }
            leader_ids
        };
        blockchain_configuration.committees = {
            let mut committees = Vec::new();
            for committee in blockchain.committees() {
                let wallet = self
                    .wallets
                    .get(&committee)
                    .expect(&format!("committee not defined {}", committee));
                committees.push(CommitteeIdDef::from(wallet.committee_id()));
            }
            committees
        };
        blockchain_configuration.slots_per_epoch = *blockchain.slots_per_epoch();
        blockchain_configuration.slot_duration = *blockchain.slot_duration();
        blockchain_configuration.treasury = Some(1_000_000.into());
        // TODO blockchain_configuration.linear_fees = ;
        blockchain_configuration.kes_update_speed = *blockchain.kes_update_speed();
        blockchain_configuration.consensus_genesis_praos_active_slot_coeff =
            ActiveSlotCoefficient::MAXIMUM;
    }

    fn populate_block0_blockchain_initials<'a, RNG, I>(
        &'a mut self,
        wallet_templates: I,
        rng: &mut Random<RNG>,
    ) where
        RNG: RngCore + CryptoRng,
        I: Iterator<Item = &'a WalletTemplate>,
    {
        for wallet_template in wallet_templates {
            // TODO: check the wallet does not already exist ?
            let wallet = match wallet_template.wallet_type() {
                WalletType::UTxO => Wallet::generate_utxo(wallet_template.clone(), rng.rng_mut()),
                WalletType::Account => {
                    Wallet::generate_account(wallet_template.clone(), rng.rng_mut())
                }
            };

            let initial_address = wallet.address();

            // TODO add support for sharing fragment with multiple utxos
            let initial_fragment = Initial::Fund(vec![InitialUTxO {
                address: initial_address,
                value: *wallet_template.value(),
            }]);

            self.wallets
                .insert(wallet_template.alias().clone(), wallet.clone());
            self.block0.initial.push(initial_fragment);

            if let Some(delegation) = wallet_template.delegate() {
                use chain_impl_mockchain::certificate::PoolId as StakePoolId;

                // 1. retrieve the public data (we may need to create a stake pool
                //    registration here)
                let stake_pool_id: StakePoolId = if let Some(node) = self.nodes.get_mut(delegation)
                {
                    if let Some(genesis) = &node.secret.genesis {
                        genesis.node_id.clone().into_digest_of()
                    } else {
                        // create and register the stake pool
                        let owner = WalletLib::new_account(&mut rand::rngs::OsRng);
                        let stake_pool = StakePool::new(&owner);
                        let node_id = stake_pool.id();
                        node.secret.genesis = Some(GenesisPraos {
                            sig_key: stake_pool.kes().signing_key(),
                            vrf_key: stake_pool.vrf().signing_key(),
                            node_id: {
                                let bytes: [u8; 32] = node_id.clone().into();
                                bytes.into()
                            },
                        });

                        self.block0
                            .initial
                            .push(Initial::Cert(signed_stake_pool_cert(&stake_pool).into()));

                        self.stake_pools
                            .insert(delegation.clone(), stake_pool.clone());

                        node_id
                    }
                } else {
                    // delegating to a node that does not exist in the topology
                    // so generate valid stake pool registration and delegation
                    // to that node.
                    unimplemented!("delegating stake to a stake pool that is not a node is not supported (yet)")
                };

                // 2. create delegation certificate for the wallet stake key
                // and add it to the block0.initial array
                let delegation_certificate = wallet.delegation_cert_for_block0(stake_pool_id);

                self.block0.initial.push(delegation_certificate);
            }
        }
    }

    fn populate_trusted_peers(&mut self) {
        //generate public id for all nodes treated as trusted peers
        let mut trusted_peers_aliases = HashSet::new();

        //gather aliases which are trusted peers
        for (_alias, node) in self.nodes.iter() {
            for trusted_peer in node.node_topology.trusted_peers() {
                trusted_peers_aliases.insert(trusted_peer.clone());
            }
        }

        //generate public id for trusted peers
        for alias in trusted_peers_aliases {
            self.nodes.get_mut(&alias).unwrap().config.p2p.public_id =
                Some(poldercast::Id::generate(rand::thread_rng()));
        }

        let nodes = self.nodes.clone();
        for (_alias, node) in self.nodes.iter_mut() {
            let mut trusted_peers = Vec::new();

            for trusted_peer in node.node_topology.trusted_peers() {
                let trusted_peer = nodes.get(trusted_peer).unwrap();
                trusted_peers.push(trusted_peer.config.p2p.make_trusted_peer_setting());
            }

            node.config.skip_bootstrap = Some(trusted_peers.is_empty());
            node.config.bootstrap_from_trusted_peers = Some(!trusted_peers.is_empty());
            node.config.p2p.trusted_peers = trusted_peers;
        }
    }
}
