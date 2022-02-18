#![warn(
    unused_extern_crates,
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::fallible_impl_from,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::dbg_macro
)]
#![forbid(unsafe_code)]

//! # judecoin-harness
//!
//! A simple lib to start a judecoin container (incl. judecoind and
//! judecoin-wallet-rpc). Provides initialisation methods to generate blocks,
//! create and fund accounts, and start a continuous mining task mining blocks
//! every BLOCK_TIME_SECS seconds.
//!
//! Also provides standalone JSON RPC clients for judecoind and judecoin-wallet-rpc.
pub mod image;

use crate::image::{JUDECOIND_DAEMON_CONTAINER_NAME, JUDECOIND_DEFAULT_NETWORK, RPC_PORT};
use anyhow::{anyhow, bail, Context, Result};
use judecoin_rpc::judecoind;
use judecoin_rpc::judecoind::judecoindRpc as _;
use judecoin_rpc::wallet::{self, GetAddress, judecoinWalletRpc as _, Refreshed, Transfer};
use std::time::Duration;
use testcontainers::clients::Cli;
use testcontainers::{Container, Docker, RunArgs};
use tokio::time;

/// How often we mine a block.
const BLOCK_TIME_SECS: u64 = 1;

/// Poll interval when checking if the wallet has synced with judecoind.
const WAIT_WALLET_SYNC_MILLIS: u64 = 1000;

#[derive(Clone, Debug)]
pub struct judecoin {
    judecoind: judecoind,
    wallets: Vec<judecoinWalletRpc>,
}
impl<'c> judecoin {
    /// Starts a new regtest judecoin container setup consisting out of 1 judecoind
    /// node and n wallets. The docker container and network will be prefixed
    /// with a randomly generated `prefix`. One miner wallet is started
    /// automatically.
    /// judecoind container name is: `prefix`_`judecoind`
    /// network is: `prefix`_`judecoin`
    /// miner wallet container name is: `miner`
    pub async fn new(
        cli: &'c Cli,
        additional_wallets: Vec<&'static str>,
    ) -> Result<(
        Self,
        Container<'c, Cli, image::judecoind>,
        Vec<Container<'c, Cli, image::judecoinWalletRpc>>,
    )> {
        let prefix = format!("{}_", random_prefix());
        let judecoind_name = format!("{}{}", prefix, JUDECOIND_DAEMON_CONTAINER_NAME);
        let network = format!("{}{}", prefix, JUDECOIND_DEFAULT_NETWORK);

        tracing::info!("Starting judecoind: {}", judecoind_name);
        let (judecoind, judecoind_container) = judecoind::new(cli, judecoind_name, network)?;
        let mut containers = vec![];
        let mut wallets = vec![];

        let miner = "miner";
        tracing::info!("Starting miner wallet: {}", miner);
        let (miner_wallet, miner_container) =
            judecoinWalletRpc::new(cli, &miner, &judecoind, prefix.clone()).await?;

        wallets.push(miner_wallet);
        containers.push(miner_container);
        for wallet in additional_wallets.iter() {
            tracing::info!("Starting wallet: {}", wallet);

            // Create new wallet, the RPC sometimes has startup problems so we allow retries
            // (drop the container that failed and try again) Times out after
            // trying for 5 minutes
            let (wallet, container) = tokio::time::timeout(Duration::from_secs(300), async {
                loop {
                    let result = judecoinWalletRpc::new(cli, &wallet, &judecoind, prefix.clone()).await;

                    match result {
                        Ok(tuple) => { return tuple; }
                        Err(e) => { tracing::warn!("judecoin wallet RPC emitted error {} - retrying to create wallet in 2 seconds...", e); }
                    }
                }
            }).await.context("All retry attempts for creating a wallet exhausted")?;

            wallets.push(wallet);
            containers.push(container);
        }

        Ok((Self { judecoind, wallets }, judecoind_container, containers))
    }

    pub fn judecoind(&self) -> &judecoind {
        &self.judecoind
    }

    pub fn wallet(&self, name: &str) -> Result<&judecoinWalletRpc> {
        let wallet = self
            .wallets
            .iter()
            .find(|wallet| wallet.name.eq(&name))
            .ok_or_else(|| anyhow!("Could not find wallet container."))?;

        Ok(wallet)
    }

    pub async fn init_miner(&self) -> Result<()> {
        let miner_wallet = self.wallet("miner")?;
        let miner_address = miner_wallet.address().await?.address;

        // generate the first 70 as bulk
        let judecoind = &self.judecoind;
        let res = judecoind
            .client()
            .generateblocks(70, miner_address.clone())
            .await?;
        tracing::info!("Generated {:?} blocks", res.blocks.len());
        miner_wallet.refresh().await?;

        Ok(())
    }

    pub async fn init_wallet(&self, name: &str, amount_in_outputs: Vec<u64>) -> Result<()> {
        let miner_wallet = self.wallet("miner")?;
        let miner_address = miner_wallet.address().await?.address;
        let judecoind = &self.judecoind;

        let wallet = self.wallet(name)?;
        let address = wallet.address().await?.address;

        for amount in amount_in_outputs {
            if amount > 0 {
                miner_wallet.transfer(&address, amount).await?;
                tracing::info!("Funded {} wallet with {}", wallet.name, amount);
                judecoind
                    .client()
                    .generateblocks(10, miner_address.clone())
                    .await?;
                wallet.refresh().await?;
            }
        }

        Ok(())
    }

    pub async fn start_miner(&self) -> Result<()> {
        let miner_wallet = self.wallet("miner")?;
        let miner_address = miner_wallet.address().await?.address;
        let judecoind = &self.judecoind;

        judecoind.start_miner(&miner_address).await?;

        tracing::info!("Waiting for miner wallet to catch up...");
        let block_height = judecoind.client().get_block_count().await?.count;
        miner_wallet
            .wait_for_wallet_height(block_height)
            .await
            .unwrap();

        Ok(())
    }

    pub async fn init_and_start_miner(&self) -> Result<()> {
        self.init_miner().await?;
        self.start_miner().await?;

        Ok(())
    }
}

fn random_prefix() -> String {
    use rand::Rng;

    rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(4)
        .collect()
}

#[derive(Clone, Debug)]
pub struct judecoind {
    rpc_port: u16,
    name: String,
    network: String,
    client: judecoind::Client,
}

#[derive(Clone, Debug)]
pub struct judecoinWalletRpc {
    rpc_port: u16,
    name: String,
    network: String,
    client: wallet::Client,
}

impl<'c> judecoind {
    /// Starts a new regtest judecoin container.
    fn new(
        cli: &'c Cli,
        name: String,
        network: String,
    ) -> Result<(Self, Container<'c, Cli, image::judecoind>)> {
        let image = image::judecoind::default();
        let run_args = RunArgs::default()
            .with_name(name.clone())
            .with_network(network.clone());
        let container = cli.run_with_args(image, run_args);
        let judecoind_rpc_port = container
            .get_host_port(RPC_PORT)
            .context("port not exposed")?;

        Ok((
            Self {
                rpc_port: judecoind_rpc_port,
                name,
                network,
                client: judecoind::Client::localhost(judecoind_rpc_port)?,
            },
            container,
        ))
    }

    pub fn client(&self) -> &judecoind::Client {
        &self.client
    }

    /// Spawns a task to mine blocks in a regular interval to the provided
    /// address
    pub async fn start_miner(&self, miner_wallet_address: &str) -> Result<()> {
        let judecoind = self.client().clone();
        let _ = tokio::spawn(mine(judecoind, miner_wallet_address.to_string()));
        Ok(())
    }
}

impl<'c> judecoinWalletRpc {
    /// Starts a new wallet container which is attached to
    /// JUDECOIND_DEFAULT_NETWORK and JUDECOIND_DAEMON_CONTAINER_NAME
    async fn new(
        cli: &'c Cli,
        name: &str,
        judecoind: &judecoind,
        prefix: String,
    ) -> Result<(Self, Container<'c, Cli, image::judecoinWalletRpc>)> {
        let daemon_address = format!("{}:{}", judecoind.name, RPC_PORT);
        let image = image::judecoinWalletRpc::new(&name, daemon_address);

        let network = judecoind.network.clone();
        let run_args = RunArgs::default()
            // prefix the container name so we can run multiple tests
            .with_name(format!("{}{}", prefix, name))
            .with_network(network.clone());
        let container = cli.run_with_args(image, run_args);
        let wallet_rpc_port = container
            .get_host_port(RPC_PORT)
            .context("port not exposed")?;

        let client = wallet::Client::localhost(wallet_rpc_port)?;

        client
            .create_wallet(name.to_owned(), "English".to_owned())
            .await?;

        Ok((
            Self {
                rpc_port: wallet_rpc_port,
                name: name.to_string(),
                network,
                client,
            },
            container,
        ))
    }

    pub fn client(&self) -> &wallet::Client {
        &self.client
    }

    // It takes a little while for the wallet to sync with judecoind.
    pub async fn wait_for_wallet_height(&self, height: u32) -> Result<()> {
        let mut retry: u8 = 0;
        while self.client().get_height().await?.height < height {
            if retry >= 30 {
                // ~30 seconds
                bail!("Wallet could not catch up with judecoind after 30 retries.")
            }
            time::sleep(Duration::from_millis(WAIT_WALLET_SYNC_MILLIS)).await;
            retry += 1;
        }
        Ok(())
    }

    /// Sends amount to address
    pub async fn transfer(&self, address: &str, amount: u64) -> Result<Transfer> {
        Ok(self.client().transfer_single(0, amount, address).await?)
    }

    pub async fn address(&self) -> Result<GetAddress> {
        Ok(self.client().get_address(0).await?)
    }

    pub async fn balance(&self) -> Result<u64> {
        self.client().refresh().await?;
        let balance = self.client().get_balance(0).await?.balance;

        Ok(balance)
    }

    pub async fn refresh(&self) -> Result<Refreshed> {
        Ok(self.client().refresh().await?)
    }
}
/// Mine a block ever BLOCK_TIME_SECS seconds.
async fn mine(judecoind: judecoind::Client, reward_address: String) -> Result<()> {
    loop {
        time::sleep(Duration::from_secs(BLOCK_TIME_SECS)).await;
        judecoind.generateblocks(1, reward_address.clone()).await?;
    }
}
