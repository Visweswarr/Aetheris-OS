use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128, Timestamp,
};

use crate::error::ContractError;
use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg};
use crate::state::{CONFIG, ADMIN, next_attestation_id, next_schema_id};
use crate::contract::{execute, instantiate, query};

// ============ ENTRY POINTS ============

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    instantiate(deps, env, info, msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    execute(deps, env, info, msg)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    query(deps, env, msg)
}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg,
) -> Result<Response, ContractError> {
    // For now, just return success
    // In the future, this could handle contract upgrades
    Ok(Response::new().add_attribute("method", "migrate"))
}

// ============ MODULE IMPORTS ============

mod contract;
mod error;
mod msg;
mod state;
mod handlers;
mod queries;
mod validation;

// ============ RE-EXPORTS ============

pub use crate::error::ContractError;
pub use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg};
