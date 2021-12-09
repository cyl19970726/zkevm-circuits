use bus_mapping::eth_types::Address;
use bus_mapping::rpc::GethClient;
use ethers::{
    abi,
    core::k256::ecdsa::SigningKey,
    core::types::Bytes,
    providers::{Http, Provider},
    signers::{coins_bip39::English, MnemonicBuilder, Signer, Wallet},
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::{self, VarError};
use std::fs::File;
use std::time::Duration;
use url::Url;

pub const CHAIN_ID: u64 = 1337;
pub const CONTRACTS_PATH: &str = "contracts";
pub const CONTRACTS: &[(&str, &str)] = &[("Greeter", "greeter/Greeter.sol")];
pub const GENDATA_OUTPUT_PATH: &str = "gendata_output.json";

const GETH0_URL_DEFAULT: &str = "http://localhost:8545";

lazy_static! {
    pub static ref GETH0_URL: String = match env::var("GETH0_URL") {
        Ok(val) => val,
        Err(VarError::NotPresent) => GETH0_URL_DEFAULT.to_string(),
        Err(e) => panic!("Error in GETH0_URL env var: {:?}", e),
    };
}

pub fn get_client() -> GethClient<Http> {
    let transport = Http::new(Url::parse(&GETH0_URL).expect("invalid url"));
    GethClient::new(transport)
}

pub fn get_provider() -> Provider<Http> {
    let transport = Http::new(Url::parse(&GETH0_URL).expect("invalid url"));
    Provider::new(transport).interval(Duration::from_millis(100))
}

const PHRASE: &str = "work man father plunge mystery proud hollow address reunion sauce theory bonus";

pub fn get_wallet(index: u32) -> Wallet<SigningKey> {
    // Access mnemonic phrase.
    // Child key at derivation path: m/44'/60'/0'/0/{index}
    MnemonicBuilder::<English>::default()
        .phrase(PHRASE)
        .index(index)
        .expect("invalid index")
        .build()
        .expect("cannot build wallet from mnemonic")
        .with_chain_id(CHAIN_ID)
}

#[derive(Serialize, Deserialize)]
pub struct GenDataOutput {
    pub coinbase: Address,
    pub wallets: Vec<Address>,
    /// Map of ContractName -> (BlockNum, Address)
    pub deployments: HashMap<String, (u64, Address)>,
}

impl GenDataOutput {
    pub fn load() -> Self {
        serde_json::from_reader(
            File::open(GENDATA_OUTPUT_PATH).expect("cannot read file"),
        )
        .expect("cannot deserialize json from file")
    }

    pub fn store(&self) {
        serde_json::to_writer(
            &File::create(GENDATA_OUTPUT_PATH).expect("cannot create file"),
            self,
        )
        .expect("cannot serialize json into file");
    }
}

#[derive(Serialize, Deserialize)]
pub struct CompiledContract {
    pub path: String,
    pub name: String,
    pub abi: abi::Contract,
    pub bin: Bytes,
    pub bin_runtime: Bytes,
}
