#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response};
use std::fmt::Debug;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

mod execute;
mod query;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, axelar_wasm_std::ContractError> {
    Ok(internal::instantiate(deps, env, info, msg)?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, axelar_wasm_std::ContractError> {
    Ok(internal::execute(deps, env, info, msg)?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
) -> Result<Binary, axelar_wasm_std::ContractError> {
    Ok(internal::query(deps, env, msg)?)
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("gateway contract config is missing")]
    ConfigMissing,
    #[error("invalid store access")]
    InvalidStoreAccess,
    #[error("failed to serialize the response")]
    SerializeResponse,
    #[error("batch contains duplicate message ids")]
    DuplicateMessageIds,
    #[error("could not query the verifier contract")]
    QueryVerifier,
    #[error("invalid address")]
    InvalidAddress,
}

mod internal {
    use crate::contract::Error;
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::router::Router;
    use crate::state::Config;
    use crate::verifier::Verifier;
    use crate::{contract, state};
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response};
    use error_stack::{Result, ResultExt};

    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, Error> {
        let router = deps
            .api
            .addr_validate(&msg.router_address)
            .change_context(Error::InvalidAddress)
            .attach_printable(msg.router_address)?;

        let verifier = deps
            .api
            .addr_validate(&msg.verifier_address)
            .change_context(Error::InvalidAddress)
            .attach_printable(msg.verifier_address)?;

        state::save_config(deps.storage, &Config { verifier, router })
            .change_context(Error::InvalidStoreAccess)?;

        Ok(Response::new())
    }

    pub(crate) fn execute(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, Error> {
        let config = state::load_config(deps.storage).change_context(Error::ConfigMissing)?;
        let verifier = Verifier {
            address: config.verifier,
            querier: deps.querier,
        };

        let router = Router {
            address: config.router,
        };

        match msg {
            ExecuteMsg::VerifyMessages(msgs) => contract::execute::verify_messages(&verifier, msgs),
            ExecuteMsg::RouteMessages(msgs) => {
                if info.sender == router.address {
                    contract::execute::route_outgoing_messages(deps.storage, msgs)
                } else {
                    contract::execute::route_incoming_messages(&verifier, &router, msgs)
                }
            }
        }
    }

    pub(crate) fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, Error> {
        match msg {
            QueryMsg::GetMessages { message_ids } => {
                contract::query::get_outgoing_messages(deps.storage, message_ids)
            }
        }
    }
}
