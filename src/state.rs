use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};

pub static CONFIG_KEY: &[u8] = b"config";
pub static LAST_WINNER_KEY: &[u8] = b"last_winner_key";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub terrand_address: CanonicalAddr,
    pub minimum_bet_amount: Uint128,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG_KEY)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LastWinner {
    pub address: CanonicalAddr,
    pub pot_amount: Uint128,
}

pub fn last_winner<S: Storage>(storage: &mut S) -> Singleton<S, LastWinner> {
    singleton(storage, LAST_WINNER_KEY)
}

pub fn last_winner_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, LastWinner> {
    singleton_read(storage, LAST_WINNER_KEY)
}