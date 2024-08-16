use std::collections::HashMap;

use axelar_wasm_std::IntoContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError, Storage};
use cw_storage_plus::{Item, Map};
use router_api::{Address, ChainName, ChainNameRaw};

#[derive(thiserror::Error, Debug, IntoContractError)]
pub enum Error {
    #[error(transparent)]
    Std(#[from] StdError),
    #[error("ITS contract got into an invalid state, its config is missing")]
    MissingConfig,
    #[error("its address for chain {0} not found")]
    ItsAddressNotFound(ChainName),
}

#[cw_serde]
pub struct Config {
    pub chain_name: ChainNameRaw,
    pub gateway: Addr,
}

const CONFIG: Item<Config> = Item::new("config");
const ITS_ADDRESSES: Map<&ChainName, Address> = Map::new("its_addresses");

pub(crate) fn load_config(storage: &dyn Storage) -> Result<Config, Error> {
    CONFIG
        .may_load(storage)
        .map_err(Error::from)?
        .ok_or(Error::MissingConfig)
}

pub(crate) fn save_config(storage: &mut dyn Storage, config: &Config) -> Result<(), Error> {
    CONFIG.save(storage, config).map_err(Error::from)
}

pub(crate) fn load_its_address(storage: &dyn Storage, chain: &ChainName) -> Result<Address, Error> {
    ITS_ADDRESSES
        .may_load(storage, chain)
        .map_err(Error::from)?
        .ok_or_else(|| Error::ItsAddressNotFound(chain.clone()))
}

pub(crate) fn save_its_address(
    storage: &mut dyn Storage,
    chain: &ChainName,
    address: &Address,
) -> Result<(), Error> {
    ITS_ADDRESSES
        .save(storage, chain, address)
        .map_err(Error::from)
}

pub(crate) fn remove_its_address(storage: &mut dyn Storage, chain: &ChainName) {
    ITS_ADDRESSES.remove(storage, chain)
}

pub(crate) fn load_all_its_addresses(
    storage: &dyn Storage,
) -> Result<HashMap<ChainName, Address>, Error> {
    ITS_ADDRESSES
        .range(storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<Result<HashMap<_, _>, _>>()
        .map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::mock_dependencies;

    use super::*;

    #[test]
    fn config_storage() {
        let mut deps = mock_dependencies();

        // Test saving and loading config
        let config = Config {
            chain_name: "test-chain".parse().unwrap(),
            gateway: Addr::unchecked("gateway-address"),
        };

        assert!(save_config(deps.as_mut().storage, &config).is_ok());
        assert_eq!(load_config(deps.as_ref().storage).unwrap(), config);

        // Test missing config
        let deps = mock_dependencies();
        assert!(matches!(
            load_config(deps.as_ref().storage),
            Err(Error::MissingConfig)
        ));
    }

    #[test]
    fn its_addresses_storage() {
        let mut deps = mock_dependencies();

        let chain = "test-chain".parse().unwrap();
        let address: Address = "its-address".parse().unwrap();

        // Test saving and loading its address
        assert!(save_its_address(deps.as_mut().storage, &chain, &address).is_ok());
        assert_eq!(
            load_its_address(deps.as_ref().storage, &chain).unwrap(),
            address
        );

        // Test removing its address
        remove_its_address(deps.as_mut().storage, &chain);
        assert!(matches!(
            load_its_address(deps.as_ref().storage, &chain),
            Err(Error::ItsAddressNotFound(_))
        ));

        // Test getting all its addresses
        let chain1 = "chain1".parse().unwrap();
        let chain2 = "chain2".parse().unwrap();
        let address1: Address = "address1".parse().unwrap();
        let address2: Address = "address2".parse().unwrap();
        assert!(save_its_address(deps.as_mut().storage, &chain1, &address1).is_ok());
        assert!(save_its_address(deps.as_mut().storage, &chain2, &address2).is_ok());

        let all_addresses = load_all_its_addresses(deps.as_ref().storage).unwrap();
        assert_eq!(
            all_addresses,
            [(chain1, address1), (chain2, address2)]
                .into_iter()
                .collect::<HashMap<_, _>>()
        );
    }
}