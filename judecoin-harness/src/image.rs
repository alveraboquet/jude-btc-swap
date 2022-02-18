use std::collections::HashMap;
use testcontainers::core::{Container, Docker, WaitForMessage};
use testcontainers::Image;

pub const JUDECOIND_DAEMON_CONTAINER_NAME: &str = "judecoind";
pub const JUDECOIND_DEFAULT_NETWORK: &str = "judecoin_network";

/// The port we use for all RPC communication.
///
/// This is the default when running judecoind.
/// For `judecoin-wallet-rpc` we always need to specify a port. To make things
/// simpler, we just specify the same one. They are in different containers so
/// this doesn't matter.
pub const RPC_PORT: u16 = 18081;

#[derive(Debug)]
pub struct judecoind {
    args: judecoindArgs,
}

impl Image for judecoind {
    type Args = judecoindArgs;
    type EnvVars = HashMap<String, String>;
    type Volumes = HashMap<String, String>;
    type EntryPoint = str;

    fn descriptor(&self) -> String {
        "judeto/judecoin:v0.17.2.0".to_owned()
    }

    fn wait_until_ready<D: Docker>(&self, container: &Container<'_, D, Self>) {
        container
            .logs()
            .stdout
            .wait_for_message("RPC server started ok")
            .unwrap();
    }

    fn args(&self) -> <Self as Image>::Args {
        self.args.clone()
    }

    fn volumes(&self) -> Self::Volumes {
        HashMap::new()
    }

    fn env_vars(&self) -> Self::EnvVars {
        HashMap::new()
    }

    fn with_args(self, args: <Self as Image>::Args) -> Self {
        Self { args }
    }

    fn entrypoint(&self) -> Option<String> {
        Some("".to_owned()) // an empty entrypoint disables the entrypoint
                            // script and gives us full control
    }
}

impl Default for judecoind {
    fn default() -> Self {
        Self {
            args: judecoindArgs::default(),
        }
    }
}

#[derive(Debug)]
pub struct judecoinWalletRpc {
    args: judecoinWalletRpcArgs,
}

impl Image for judecoinWalletRpc {
    type Args = judecoinWalletRpcArgs;
    type EnvVars = HashMap<String, String>;
    type Volumes = HashMap<String, String>;
    type EntryPoint = str;

    fn descriptor(&self) -> String {
        "judeto/judecoin:v0.17.2.0".to_owned()
    }

    fn wait_until_ready<D: Docker>(&self, container: &Container<'_, D, Self>) {
        container
            .logs()
            .stdout
            .wait_for_message("Run server thread name: RPC")
            .unwrap();
    }

    fn args(&self) -> <Self as Image>::Args {
        self.args.clone()
    }

    fn volumes(&self) -> Self::Volumes {
        HashMap::new()
    }

    fn env_vars(&self) -> Self::EnvVars {
        HashMap::new()
    }

    fn with_args(self, args: <Self as Image>::Args) -> Self {
        Self { args }
    }

    fn entrypoint(&self) -> Option<String> {
        Some("".to_owned()) // an empty entrypoint disables the entrypoint
                            // script and gives us full control
    }
}

impl Default for judecoinWalletRpc {
    fn default() -> Self {
        Self {
            args: judecoinWalletRpcArgs::default(),
        }
    }
}

impl judecoinWalletRpc {
    pub fn new(name: &str, daemon_address: String) -> Self {
        Self {
            args: judecoinWalletRpcArgs::new(name, daemon_address),
        }
    }
}

#[derive(Debug, Clone)]
pub struct judecoindArgs {
    pub regtest: bool,
    pub offline: bool,
    pub rpc_payment_allow_free_loopback: bool,
    pub confirm_external_bind: bool,
    pub no_igd: bool,
    pub hide_my_port: bool,
    pub rpc_bind_ip: String,
    pub fixed_difficulty: u32,
    pub data_dir: String,
}

impl Default for judecoindArgs {
    fn default() -> Self {
        Self {
            regtest: true,
            offline: true,
            rpc_payment_allow_free_loopback: true,
            confirm_external_bind: true,
            no_igd: true,
            hide_my_port: true,
            rpc_bind_ip: "0.0.0.0".to_string(),
            fixed_difficulty: 1,
            data_dir: "/judecoin".to_string(),
        }
    }
}

impl IntoIterator for judecoindArgs {
    type Item = String;
    type IntoIter = ::std::vec::IntoIter<String>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        let mut args = vec![
            "judecoind".to_string(),
            "--log-level=4".to_string(),
            "--non-interactive".to_string(),
        ];

        if self.regtest {
            args.push("--regtest".to_string())
        }

        if self.offline {
            args.push("--offline".to_string())
        }

        if self.rpc_payment_allow_free_loopback {
            args.push("--rpc-payment-allow-free-loopback".to_string())
        }

        if self.confirm_external_bind {
            args.push("--confirm-external-bind".to_string())
        }

        if self.no_igd {
            args.push("--no-igd".to_string())
        }

        if self.hide_my_port {
            args.push("--hide-my-port".to_string())
        }

        if !self.rpc_bind_ip.is_empty() {
            args.push(format!("--rpc-bind-ip={}", self.rpc_bind_ip));
        }

        if !self.data_dir.is_empty() {
            args.push(format!("--data-dir={}", self.data_dir));
        }

        if self.fixed_difficulty != 0 {
            args.push(format!("--fixed-difficulty={}", self.fixed_difficulty));
        }

        args.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct judecoinWalletRpcArgs {
    pub disable_rpc_login: bool,
    pub confirm_external_bind: bool,
    pub wallet_dir: String,
    pub rpc_bind_ip: String,
    pub daemon_address: String,
}

impl Default for judecoinWalletRpcArgs {
    fn default() -> Self {
        unimplemented!("A default instance for `judecoinWalletRpc` doesn't make sense because we always need to connect to a node.")
    }
}

impl judecoinWalletRpcArgs {
    pub fn new(wallet_name: &str, daemon_address: String) -> Self {
        Self {
            disable_rpc_login: true,
            confirm_external_bind: true,
            wallet_dir: wallet_name.into(),
            rpc_bind_ip: "0.0.0.0".into(),
            daemon_address,
        }
    }
}

impl IntoIterator for judecoinWalletRpcArgs {
    type Item = String;
    type IntoIter = ::std::vec::IntoIter<String>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        let mut args = vec![
            "judecoin-wallet-rpc".to_string(),
            format!("--wallet-dir={}", self.wallet_dir),
            format!("--daemon-address={}", self.daemon_address),
            format!("--rpc-bind-port={}", RPC_PORT),
            "--log-level=4".to_string(),
        ];

        if self.disable_rpc_login {
            args.push("--disable-rpc-login".to_string())
        }

        if self.confirm_external_bind {
            args.push("--confirm-external-bind".to_string())
        }

        if !self.rpc_bind_ip.is_empty() {
            args.push(format!("--rpc-bind-ip={}", self.rpc_bind_ip));
        }

        args.into_iter()
    }
}
