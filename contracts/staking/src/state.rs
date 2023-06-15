// Importing necessary dependencies
use std::any::type_name;

use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use cosmwasm_std::{
    from_slice,
    storage_keys::{namespace_with_key, to_length_prefixed},
    to_vec, Addr, CanonicalAddr, Decimal, StdError, StdResult, Storage, Uint128,
};

// Constants
pub const KEY_INVESTMENT: &[u8] = b"invest";
pub const KEY_TOKEN_INFO: &[u8] = b"token";
pub const KEY_TOTAL_SUPPLY: &[u8] = b"total_supply";

pub const PREFIX_BALANCE: &[u8] = b"balance";
pub const PREFIX_CLAIMS: &[u8] = b"claim";

// Functions to handle map loading and saving in storage
// These functions abstract out the common pattern of accessing the storage
pub fn may_load_map(
    storage: &dyn Storage,
    prefix: &[u8],
    key: &CanonicalAddr,
) -> StdResult<Option<Uint128>> {
    storage
        .get(&namespace_with_key(&[prefix], key))
        .map(|v| from_slice(&v))
        .transpose()
}

pub fn save_map(
    storage: &mut dyn Storage,
    prefix: &[u8],
    key: &CanonicalAddr,
    value: Uint128,
) -> StdResult<()> {
    storage.set(&namespace_with_key(&[prefix], key), &to_vec(&value)?);
    Ok(())
}

pub fn load_map(storage: &dyn Storage, prefix: &[u8], key: &CanonicalAddr) -> StdResult<Uint128> {
    may_load_map(storage, prefix, key)?
        .ok_or_else(|| StdError::not_found(format!("Value not found for key: {}", key)))
}

// Data structures
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InvestmentInfo {
    pub owner: Addr,
    pub bond_denom: String,
    pub exit_tax: Decimal,
    pub validator: String,
    pub min_withdrawal: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default, JsonSchema)]
pub struct Supply {
    pub issued: Uint128,
    pub bonded: Uint128,
    pub claims: Uint128,
}

// Functions to handle item loading, saving, and updating in storage
// These functions also abstract out the common pattern of accessing the storage
pub fn load_item<T: DeserializeOwned>(storage: &dyn Storage, key: &[u8]) -> StdResult<T> {
    storage
        .get(&to_length_prefixed(key))
        .ok_or_else(|| StdError::not_found(type_name::<T>()))
        .and_then(|v| from_slice(&v))
}

pub fn save_item<T: Serialize>(storage: &mut dyn Storage, key: &[u8], item: &T) -> StdResult<()> {
    storage.set(&to_length_prefixed(key), &to_vec(item)?);
    Ok(())
}

pub fn update_item<T, A, E>(storage: &mut dyn Storage, key: &[u8], action: A) -> Result<T, E>
where
    T: Serialize + DeserializeOwned,
    A: FnOnce(T) -> Result<T, E>,
    E: From<StdError>,
{
    let input = load_item(storage, key)?;
    let output = action(input)?;
    save_item(storage, key, &output)?;
    Ok(output)
}
