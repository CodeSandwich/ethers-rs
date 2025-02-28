use crate::{Client, EtherscanError, Query, Response, Result};
use ethers_core::{
    abi::Address,
    types::{serde_helpers::*, BlockNumber, Bytes, H256, H32, U256},
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{Display, Error, Formatter},
};

/// The raw response from the balance-related API endpoints
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountBalance {
    pub account: Address,
    pub balance: String,
}

mod genesis_string {
    use super::*;
    use serde::{
        de::{DeserializeOwned, Error as _},
        ser::Error as _,
        Deserializer, Serializer,
    };

    pub fn serialize<T, S>(
        value: &GenesisOption<T>,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        T: Serialize,
        S: Serializer,
    {
        let json = match value {
            GenesisOption::None => Cow::from(""),
            GenesisOption::Genesis => Cow::from("GENESIS"),
            GenesisOption::Some(value) => {
                serde_json::to_string(value).map_err(S::Error::custom)?.into()
            }
        };
        serializer.serialize_str(&json)
    }

    pub fn deserialize<'de, T, D>(
        deserializer: D,
    ) -> std::result::Result<GenesisOption<T>, D::Error>
    where
        T: DeserializeOwned,
        D: Deserializer<'de>,
    {
        let json = Cow::<'de, str>::deserialize(deserializer)?;
        if !json.is_empty() && !json.starts_with("GENESIS") {
            serde_json::from_str(&format!("\"{}\"", &json))
                .map(GenesisOption::Some)
                .map_err(D::Error::custom)
        } else if json.starts_with("GENESIS") {
            Ok(GenesisOption::Genesis)
        } else {
            Ok(GenesisOption::None)
        }
    }
}

mod json_string {
    use super::*;
    use serde::{
        de::{DeserializeOwned, Error as _},
        ser::Error as _,
        Deserializer, Serializer,
    };

    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        T: Serialize,
        S: Serializer,
    {
        let json = match value {
            Option::None => Cow::from(""),
            Option::Some(value) => serde_json::to_string(value).map_err(S::Error::custom)?.into(),
        };
        serializer.serialize_str(&json)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> std::result::Result<Option<T>, D::Error>
    where
        T: DeserializeOwned,
        D: Deserializer<'de>,
    {
        let json = Cow::<'de, str>::deserialize(deserializer)?;
        if json.is_empty() {
            Ok(Option::None)
        } else {
            serde_json::from_str(&format!("\"{}\"", &json))
                .map(Option::Some)
                .map_err(D::Error::custom)
        }
    }
}

mod hex_string {
    use super::*;
    use serde::{
        de::{DeserializeOwned, Error as _},
        ser::Error as _,
        Deserializer, Serializer,
    };

    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        T: Serialize,
        S: Serializer,
    {
        let json = match value {
            Option::None => Cow::from("0x"),
            Option::Some(value) => serde_json::to_string(value).map_err(S::Error::custom)?.into(),
        };
        serializer.serialize_str(&json)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> std::result::Result<Option<T>, D::Error>
    where
        T: DeserializeOwned,
        D: Deserializer<'de>,
    {
        let json = Cow::<'de, str>::deserialize(deserializer)?;
        if json.is_empty() || json == "0x" {
            Ok(Option::None)
        } else {
            serde_json::from_str(&format!("\"{}\"", &json))
                .map(Option::Some)
                .map_err(D::Error::custom)
        }
    }
}

/// Possible values for some field responses.
///
/// Transactions from the Genesis block may contain fields that do not conform to the expected
/// types.
#[derive(Clone, Debug)]
pub enum GenesisOption<T> {
    None,
    Genesis,
    Some(T),
}

impl<T> From<GenesisOption<T>> for Option<T> {
    fn from(value: GenesisOption<T>) -> Self {
        match value {
            GenesisOption::Some(value) => Some(value),
            _ => None,
        }
    }
}

impl<T> GenesisOption<T> {
    pub fn is_genesis(&self) -> bool {
        matches!(self, GenesisOption::Genesis)
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            GenesisOption::Some(value) => Some(value),
            _ => None,
        }
    }
}

/// The raw response from the transaction list API endpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalTransaction {
    pub is_error: String,
    #[serde(deserialize_with = "deserialize_stringified_block_number")]
    pub block_number: BlockNumber,
    pub time_stamp: String,
    #[serde(with = "genesis_string")]
    pub hash: GenesisOption<H256>,
    #[serde(with = "json_string")]
    pub nonce: Option<U256>,
    #[serde(with = "json_string")]
    pub block_hash: Option<U256>,
    #[serde(deserialize_with = "deserialize_stringified_u64_opt")]
    pub transaction_index: Option<u64>,
    #[serde(with = "genesis_string")]
    pub from: GenesisOption<Address>,
    #[serde(with = "json_string")]
    pub to: Option<Address>,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub value: U256,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub gas: U256,
    #[serde(deserialize_with = "deserialize_stringified_numeric_opt")]
    pub gas_price: Option<U256>,
    #[serde(rename = "txreceipt_status")]
    pub tx_receipt_status: String,
    pub input: Bytes,
    #[serde(with = "json_string")]
    pub contract_address: Option<Address>,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub gas_used: U256,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub cumulative_gas_used: U256,
    #[serde(deserialize_with = "deserialize_stringified_u64")]
    pub confirmations: u64,
    #[serde(with = "hex_string")]
    pub method_id: Option<H32>,
    #[serde(with = "json_string")]
    pub function_name: Option<String>,
}

/// The raw response from the internal transaction list API endpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InternalTransaction {
    #[serde(deserialize_with = "deserialize_stringified_block_number")]
    pub block_number: BlockNumber,
    pub time_stamp: String,
    pub hash: H256,
    pub from: Address,
    #[serde(with = "genesis_string")]
    pub to: GenesisOption<Address>,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub value: U256,
    #[serde(with = "genesis_string")]
    pub contract_address: GenesisOption<Address>,
    #[serde(with = "genesis_string")]
    pub input: GenesisOption<Bytes>,
    #[serde(rename = "type")]
    pub result_type: String,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub gas: U256,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub gas_used: U256,
    pub trace_id: String,
    pub is_error: String,
    pub err_code: String,
}

/// The raw response from the ERC20 transfer list API endpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ERC20TokenTransferEvent {
    #[serde(deserialize_with = "deserialize_stringified_block_number")]
    pub block_number: BlockNumber,
    pub time_stamp: String,
    pub hash: H256,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub nonce: U256,
    pub block_hash: H256,
    pub from: Address,
    pub contract_address: Address,
    pub to: Option<Address>,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub value: U256,
    pub token_name: String,
    pub token_symbol: String,
    pub token_decimal: String,
    #[serde(deserialize_with = "deserialize_stringified_u64")]
    pub transaction_index: u64,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub gas: U256,
    #[serde(deserialize_with = "deserialize_stringified_numeric_opt")]
    pub gas_price: Option<U256>,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub gas_used: U256,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub cumulative_gas_used: U256,
    /// deprecated
    pub input: String,
    #[serde(deserialize_with = "deserialize_stringified_u64")]
    pub confirmations: u64,
}

/// The raw response from the ERC721 transfer list API endpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ERC721TokenTransferEvent {
    #[serde(deserialize_with = "deserialize_stringified_block_number")]
    pub block_number: BlockNumber,
    pub time_stamp: String,
    pub hash: H256,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub nonce: U256,
    pub block_hash: H256,
    pub from: Address,
    pub contract_address: Address,
    pub to: Option<Address>,
    #[serde(rename = "tokenID")]
    pub token_id: String,
    pub token_name: String,
    pub token_symbol: String,
    pub token_decimal: String,
    #[serde(deserialize_with = "deserialize_stringified_u64")]
    pub transaction_index: u64,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub gas: U256,
    #[serde(deserialize_with = "deserialize_stringified_numeric_opt")]
    pub gas_price: Option<U256>,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub gas_used: U256,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub cumulative_gas_used: U256,
    /// deprecated
    pub input: String,
    #[serde(deserialize_with = "deserialize_stringified_u64")]
    pub confirmations: u64,
}

/// The raw response from the ERC1155 transfer list API endpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ERC1155TokenTransferEvent {
    #[serde(deserialize_with = "deserialize_stringified_block_number")]
    pub block_number: BlockNumber,
    pub time_stamp: String,
    pub hash: H256,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub nonce: U256,
    pub block_hash: H256,
    pub from: Address,
    pub contract_address: Address,
    pub to: Option<Address>,
    #[serde(rename = "tokenID")]
    pub token_id: String,
    pub token_value: String,
    pub token_name: String,
    pub token_symbol: String,
    #[serde(deserialize_with = "deserialize_stringified_u64")]
    pub transaction_index: u64,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub gas: U256,
    #[serde(deserialize_with = "deserialize_stringified_numeric_opt")]
    pub gas_price: Option<U256>,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub gas_used: U256,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub cumulative_gas_used: U256,
    /// deprecated
    pub input: String,
    #[serde(deserialize_with = "deserialize_stringified_u64")]
    pub confirmations: u64,
}

/// The raw response from the mined blocks API endpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinedBlock {
    #[serde(deserialize_with = "deserialize_stringified_block_number")]
    pub block_number: BlockNumber,
    pub time_stamp: String,
    pub block_reward: String,
}

/// The pre-defined block parameter for balance API endpoints
#[derive(Clone, Copy, Debug, Default)]
pub enum Tag {
    Earliest,
    Pending,
    #[default]
    Latest,
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Tag::Earliest => write!(f, "earliest"),
            Tag::Pending => write!(f, "pending"),
            Tag::Latest => write!(f, "latest"),
        }
    }
}

/// The list sorting preference
#[derive(Clone, Copy, Debug)]
pub enum Sort {
    Asc,
    Desc,
}

impl Display for Sort {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Sort::Asc => write!(f, "asc"),
            Sort::Desc => write!(f, "desc"),
        }
    }
}

/// Common optional arguments for the transaction or event list API endpoints
#[derive(Clone, Copy, Debug)]
pub struct TxListParams {
    start_block: u64,
    end_block: u64,
    page: u64,
    offset: u64,
    sort: Sort,
}

impl TxListParams {
    pub fn new(start_block: u64, end_block: u64, page: u64, offset: u64, sort: Sort) -> Self {
        Self { start_block, end_block, page, offset, sort }
    }
}

impl Default for TxListParams {
    fn default() -> Self {
        Self { start_block: 0, end_block: 99999999, page: 0, offset: 10000, sort: Sort::Asc }
    }
}

impl From<TxListParams> for HashMap<&'static str, String> {
    fn from(tx_params: TxListParams) -> Self {
        let mut params = HashMap::new();
        params.insert("startBlock", tx_params.start_block.to_string());
        params.insert("endBlock", tx_params.end_block.to_string());
        params.insert("page", tx_params.page.to_string());
        params.insert("offset", tx_params.offset.to_string());
        params.insert("sort", tx_params.sort.to_string());
        params
    }
}

/// Options for querying internal transactions
#[derive(Clone, Debug)]
pub enum InternalTxQueryOption {
    ByAddress(Address),
    ByTransactionHash(H256),
    ByBlockRange,
}

/// Options for querying ERC20 or ERC721 token transfers
#[derive(Clone, Debug)]
pub enum TokenQueryOption {
    ByAddress(Address),
    ByContract(Address),
    ByAddressAndContract(Address, Address),
}

impl TokenQueryOption {
    pub fn into_params(self, list_params: TxListParams) -> HashMap<&'static str, String> {
        let mut params: HashMap<&'static str, String> = list_params.into();
        match self {
            TokenQueryOption::ByAddress(address) => {
                params.insert("address", format!("{address:?}"));
                params
            }
            TokenQueryOption::ByContract(contract) => {
                params.insert("contractaddress", format!("{contract:?}"));
                params
            }
            TokenQueryOption::ByAddressAndContract(address, contract) => {
                params.insert("address", format!("{address:?}"));
                params.insert("contractaddress", format!("{contract:?}"));
                params
            }
        }
    }
}

/// The pre-defined block type for retrieving mined blocks
#[derive(Copy, Clone, Debug, Default)]
pub enum BlockType {
    #[default]
    CanonicalBlocks,
    Uncles,
}

impl Display for BlockType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            BlockType::CanonicalBlocks => write!(f, "blocks"),
            BlockType::Uncles => write!(f, "uncles"),
        }
    }
}

impl Client {
    /// Returns the Ether balance of a given address.
    ///
    /// ```no_run
    /// # use ethers_etherscan::Client;
    /// # use ethers_core::types::Chain;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let client = Client::new(Chain::Mainnet, "API_KEY").unwrap();
    ///     let balance = client
    ///         .get_ether_balance_single(&"0x58eB28A67731c570Ef827C365c89B5751F9E6b0a".parse().unwrap(),
    ///         None).await.unwrap();
    /// # }
    /// ```
    pub async fn get_ether_balance_single(
        &self,
        address: &Address,
        tag: Option<Tag>,
    ) -> Result<AccountBalance> {
        let tag_str = tag.unwrap_or_default().to_string();
        let addr_str = format!("{address:?}");
        let query = self.create_query(
            "account",
            "balance",
            HashMap::from([("address", &addr_str), ("tag", &tag_str)]),
        );
        let response: Response<String> = self.get_json(&query).await?;

        match response.status.as_str() {
            "0" => Err(EtherscanError::BalanceFailed),
            "1" => Ok(AccountBalance { account: *address, balance: response.result }),
            err => Err(EtherscanError::BadStatusCode(err.to_string())),
        }
    }

    /// Returns the balance of the accounts from a list of addresses.
    ///
    /// ```no_run
    /// # use ethers_etherscan::Client;
    /// # use ethers_core::types::Chain;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let client = Client::new(Chain::Mainnet, "API_KEY").unwrap();
    ///     let balances = client
    ///         .get_ether_balance_multi(&vec![&"0x58eB28A67731c570Ef827C365c89B5751F9E6b0a".parse().unwrap()],
    ///         None).await.unwrap();
    /// # }
    /// ```
    pub async fn get_ether_balance_multi(
        &self,
        addresses: &[&Address],
        tag: Option<Tag>,
    ) -> Result<Vec<AccountBalance>> {
        let tag_str = tag.unwrap_or_default().to_string();
        let addrs = addresses.iter().map(|x| format!("{x:?}")).collect::<Vec<String>>().join(",");
        let query: Query<HashMap<&str, &str>> = self.create_query(
            "account",
            "balancemulti",
            HashMap::from([("address", addrs.as_ref()), ("tag", tag_str.as_ref())]),
        );
        let response: Response<Vec<AccountBalance>> = self.get_json(&query).await?;

        match response.status.as_str() {
            "0" => Err(EtherscanError::BalanceFailed),
            "1" => Ok(response.result),
            err => Err(EtherscanError::BadStatusCode(err.to_string())),
        }
    }

    /// Returns the list of transactions performed by an address, with optional pagination.
    ///
    /// ```no_run
    /// # use ethers_etherscan::Client;
    /// # use ethers_core::types::Chain;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let client = Client::new(Chain::Mainnet, "API_KEY").unwrap();
    ///     let txs = client
    ///         .get_transactions(&"0x58eB28A67731c570Ef827C365c89B5751F9E6b0a".parse().unwrap(),
    ///         None).await.unwrap();
    /// # }
    /// ```
    pub async fn get_transactions(
        &self,
        address: &Address,
        params: Option<TxListParams>,
    ) -> Result<Vec<NormalTransaction>> {
        let mut tx_params: HashMap<&str, String> = params.unwrap_or_default().into();
        tx_params.insert("address", format!("{address:?}"));
        let query = self.create_query("account", "txlist", tx_params);
        let response: Response<Vec<NormalTransaction>> = self.get_json(&query).await?;

        Ok(response.result)
    }

    /// Returns the list of internal transactions performed by an address or within a transaction,
    /// with optional pagination.
    ///
    /// ```no_run
    /// # use ethers_etherscan::{Client, account::InternalTxQueryOption};
    /// # use ethers_core::types::Chain;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let client = Client::new(Chain::Mainnet, "API_KEY").unwrap();
    ///     let txs = client
    ///         .get_internal_transactions(
    ///             InternalTxQueryOption::ByAddress(
    ///                 "0x2c1ba59d6f58433fb1eaee7d20b26ed83bda51a3".parse().unwrap()), None).await.unwrap();
    /// # }
    /// ```
    pub async fn get_internal_transactions(
        &self,
        tx_query_option: InternalTxQueryOption,
        params: Option<TxListParams>,
    ) -> Result<Vec<InternalTransaction>> {
        let mut tx_params: HashMap<&str, String> = params.unwrap_or_default().into();
        match tx_query_option {
            InternalTxQueryOption::ByAddress(address) => {
                tx_params.insert("address", format!("{address:?}"));
            }
            InternalTxQueryOption::ByTransactionHash(tx_hash) => {
                tx_params.insert("txhash", format!("{tx_hash:?}"));
            }
            _ => {}
        }
        let query = self.create_query("account", "txlistinternal", tx_params);
        let response: Response<Vec<InternalTransaction>> = self.get_json(&query).await?;

        Ok(response.result)
    }

    /// Returns the list of ERC-20 tokens transferred by an address, with optional filtering by
    /// token contract.
    ///
    /// ```no_run
    /// # use ethers_etherscan::{Client, account::TokenQueryOption};
    /// # use ethers_core::types::Chain;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let client = Client::new(Chain::Mainnet, "API_KEY").unwrap();
    ///     let txs = client
    ///         .get_erc20_token_transfer_events(
    ///             TokenQueryOption::ByAddress(
    ///                 "0x4e83362442b8d1bec281594cea3050c8eb01311c".parse().unwrap()), None).await.unwrap();
    /// # }
    /// ```
    pub async fn get_erc20_token_transfer_events(
        &self,
        event_query_option: TokenQueryOption,
        params: Option<TxListParams>,
    ) -> Result<Vec<ERC20TokenTransferEvent>> {
        let params = event_query_option.into_params(params.unwrap_or_default());
        let query = self.create_query("account", "tokentx", params);
        let response: Response<Vec<ERC20TokenTransferEvent>> = self.get_json(&query).await?;

        Ok(response.result)
    }

    /// Returns the list of ERC-721 ( NFT ) tokens transferred by an address, with optional
    /// filtering by token contract.
    ///
    /// ```no_run
    /// # use ethers_etherscan::{Client, account::TokenQueryOption};
    /// # use ethers_core::types::Chain;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let client = Client::new(Chain::Mainnet, "API_KEY").unwrap();
    ///     let txs = client
    ///         .get_erc721_token_transfer_events(
    ///             TokenQueryOption::ByAddressAndContract(
    ///                 "0x6975be450864c02b4613023c2152ee0743572325".parse().unwrap(),
    ///                 "0x06012c8cf97bead5deae237070f9587f8e7a266d".parse().unwrap(),
    ///          ), None).await.unwrap();
    /// # }
    /// ```
    pub async fn get_erc721_token_transfer_events(
        &self,
        event_query_option: TokenQueryOption,
        params: Option<TxListParams>,
    ) -> Result<Vec<ERC721TokenTransferEvent>> {
        let params = event_query_option.into_params(params.unwrap_or_default());
        let query = self.create_query("account", "tokennfttx", params);
        let response: Response<Vec<ERC721TokenTransferEvent>> = self.get_json(&query).await?;

        Ok(response.result)
    }

    /// Returns the list of ERC-1155 ( NFT ) tokens transferred by an address, with optional
    /// filtering by token contract.
    ///
    /// ```no_run
    /// # use ethers_etherscan::{Client, account::TokenQueryOption};
    /// # use ethers_core::types::Chain;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let client = Client::new(Chain::Mainnet, "API_KEY").unwrap();
    ///     let txs = client
    ///         .get_erc1155_token_transfer_events(
    ///             TokenQueryOption::ByAddressAndContract(
    ///                 "0x216CD350a4044e7016f14936663e2880Dd2A39d7".parse().unwrap(),
    ///                 "0x495f947276749ce646f68ac8c248420045cb7b5e".parse().unwrap(),
    ///          ), None).await.unwrap();
    /// # }
    /// ```
    pub async fn get_erc1155_token_transfer_events(
        &self,
        event_query_option: TokenQueryOption,
        params: Option<TxListParams>,
    ) -> Result<Vec<ERC1155TokenTransferEvent>> {
        let params = event_query_option.into_params(params.unwrap_or_default());
        let query = self.create_query("account", "token1155tx", params);
        let response: Response<Vec<ERC1155TokenTransferEvent>> = self.get_json(&query).await?;

        Ok(response.result)
    }

    /// Returns the list of blocks mined by an address.
    ///
    /// ```no_run
    /// # use ethers_etherscan::Client;
    /// # use ethers_core::types::Chain;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let client = Client::new(Chain::Mainnet, "API_KEY").unwrap();
    ///     let blocks = client
    ///         .get_mined_blocks(&"0x9dd134d14d1e65f84b706d6f205cd5b1cd03a46b".parse().unwrap(), None, None)
    ///         .await.unwrap();
    /// # }
    /// ```
    pub async fn get_mined_blocks(
        &self,
        address: &Address,
        block_type: Option<BlockType>,
        page_and_offset: Option<(u64, u64)>,
    ) -> Result<Vec<MinedBlock>> {
        let mut params = HashMap::new();
        params.insert("address", format!("{address:?}"));
        params.insert("blocktype", block_type.unwrap_or_default().to_string());
        if let Some((page, offset)) = page_and_offset {
            params.insert("page", page.to_string());
            params.insert("offset", offset.to_string());
        }
        let query = self.create_query("account", "getminedblocks", params);
        let response: Response<Vec<MinedBlock>> = self.get_json(&query).await?;

        Ok(response.result)
    }
}
