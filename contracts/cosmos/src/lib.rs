use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128, CosmosMsg, BankMsg, Coin,
};
use cw2::set_contract_version;
use cw20_base::{
    contract::{execute as cw20_execute, instantiate as cw20_instantiate, query as cw20_query},
    msg::{ExecuteMsg as Cw20ExecuteMsg, InstantiateMsg as Cw20InstantiateMsg, QueryMsg as Cw20QueryMsg},
    state::Cw20Contract,
    ContractError as Cw20ContractError,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, ADMIN};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:polymera-cosmwasm";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Store admin
    ADMIN.save(deps.storage, &info.sender)?;

    // Store config
    let config = Config {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: msg.initial_supply,
        max_supply: msg.max_supply,
        mint_price: msg.mint_price,
        minting_enabled: true,
    };
    CONFIG.save(deps.storage, &config)?;

    // Initialize CW20 base contract
    let cw20_msg = Cw20InstantiateMsg {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        initial_balances: vec![cw20::Cw20Coin {
            address: info.sender.to_string(),
            amount: msg.initial_supply,
        }],
        mint: Some(cw20::MinterResponse {
            minter: env.contract.address.to_string(),
            cap: Some(msg.max_supply),
        }),
        marketing: None,
    };

    cw20_instantiate(deps, env, info, cw20_msg)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", info.sender)
        .add_attribute("total_supply", msg.initial_supply))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint { to, amount } => execute_mint(deps, env, info, to, amount),
        ExecuteMsg::Burn { amount } => execute_burn(deps, env, info, amount),
        ExecuteMsg::SetMintingEnabled { enabled } => execute_set_minting_enabled(deps, env, info, enabled),
        ExecuteMsg::SetMintPrice { price } => execute_set_mint_price(deps, env, info, price),
        ExecuteMsg::SetMaxSupply { max_supply } => execute_set_max_supply(deps, env, info, max_supply),
        ExecuteMsg::Withdraw {} => execute_withdraw(deps, env, info),
        ExecuteMsg::Cw20(msg) => execute_cw20(deps, env, info, msg),
    }
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    to: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    // Check if minting is enabled
    if !config.minting_enabled {
        return Err(ContractError::MintingDisabled {});
    }

    // Check if max supply would be exceeded
    let current_supply = cw20_query::<Cw20QueryMsg, _>(deps.as_ref(), Cw20QueryMsg::TokenInfo {})?;
    if let Ok(token_info) = current_supply {
        if token_info.total_supply + amount > config.max_supply {
            return Err(ContractError::MaxSupplyExceeded {});
        }
    }

    // Check if sufficient payment was sent
    let required_payment = config.mint_price * amount;
    let sent_payment = info.funds.iter().find(|coin| coin.denom == "uatom").map(|c| c.amount).unwrap_or_default();
    
    if sent_payment < required_payment {
        return Err(ContractError::InsufficientPayment { required: required_payment, sent: sent_payment });
    }

    // Execute mint through CW20 base contract
    let mint_msg = Cw20ExecuteMsg::Mint {
        recipient: to.clone(),
        amount,
    };

    cw20_execute(deps, env, info, mint_msg)?;

    Ok(Response::new()
        .add_attribute("method", "mint")
        .add_attribute("to", to)
        .add_attribute("amount", amount))
}

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // Execute burn through CW20 base contract
    let burn_msg = Cw20ExecuteMsg::Burn { amount };

    cw20_execute(deps, env, info, burn_msg)?;

    Ok(Response::new()
        .add_attribute("method", "burn")
        .add_attribute("from", info.sender)
        .add_attribute("amount", amount))
}

pub fn execute_set_minting_enabled(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    enabled: bool,
) -> Result<Response, ContractError> {
    // Check if caller is admin
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::Unauthorized {});
    }

    // Update config
    let mut config = CONFIG.load(deps.storage)?;
    config.minting_enabled = enabled;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "set_minting_enabled")
        .add_attribute("enabled", enabled.to_string()))
}

pub fn execute_set_mint_price(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    price: Uint128,
) -> Result<Response, ContractError> {
    // Check if caller is admin
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::Unauthorized {});
    }

    // Update config
    let mut config = CONFIG.load(deps.storage)?;
    config.mint_price = price;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "set_mint_price")
        .add_attribute("price", price))
}

pub fn execute_set_max_supply(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    max_supply: Uint128,
) -> Result<Response, ContractError> {
    // Check if caller is admin
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::Unauthorized {});
    }

    // Check if new max supply is less than current supply
    let current_supply = cw20_query::<Cw20QueryMsg, _>(deps.as_ref(), Cw20QueryMsg::TokenInfo {})?;
    if let Ok(token_info) = current_supply {
        if max_supply < token_info.total_supply {
            return Err(ContractError::MaxSupplyTooLow { current: token_info.total_supply, new: max_supply });
        }
    }

    // Update config
    let mut config = CONFIG.load(deps.storage)?;
    config.max_supply = max_supply;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "set_max_supply")
        .add_attribute("max_supply", max_supply))
}

pub fn execute_withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Check if caller is admin
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::Unauthorized {});
    }

    // Get contract balance
    let balance = deps.querier.query_balance(&_env.contract.address, "uatom")?;
    
    if balance.amount == Uint128::zero() {
        return Err(ContractError::NoBalanceToWithdraw {});
    }

    // Create withdrawal message
    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: admin.to_string(),
        amount: vec![balance],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("method", "withdraw")
        .add_attribute("amount", balance.amount)
        .add_attribute("recipient", admin))
}

pub fn execute_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw20ExecuteMsg,
) -> Result<Response, ContractError> {
    // Delegate to CW20 base contract for standard operations
    cw20_execute(deps, env, info, msg).map_err(ContractError::from)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Admin {} => to_binary(&query_admin(deps)?),
        QueryMsg::Cw20(msg) => query_cw20(deps, msg),
    }
}

pub fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_admin(deps: Deps) -> StdResult<String> {
    let admin = ADMIN.load(deps.storage)?;
    Ok(admin.to_string())
}

pub fn query_cw20(deps: Deps, msg: Cw20QueryMsg) -> StdResult<Binary> {
    // Delegate to CW20 base contract for standard queries
    cw20_query(deps, msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "uatom"));

        let msg = InstantiateMsg {
            name: "Polymera Token".to_string(),
            symbol: "POLY".to_string(),
            decimals: 6,
            initial_supply: Uint128::new(1000000),
            max_supply: Uint128::new(10000000),
            mint_price: Uint128::new(1000),
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.attributes.len(), 4);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");
    }

    #[test]
    fn test_query_config() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "uatom"));

        let msg = InstantiateMsg {
            name: "Polymera Token".to_string(),
            symbol: "POLY".to_string(),
            decimals: 6,
            initial_supply: Uint128::new(1000000),
            max_supply: Uint128::new(10000000),
            mint_price: Uint128::new(1000),
        };

        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let res = query_config(deps.as_ref()).unwrap();
        assert_eq!(res.name, "Polymera Token");
        assert_eq!(res.symbol, "POLY");
        assert_eq!(res.decimals, 6);
        assert_eq!(res.total_supply, Uint128::new(1000000));
        assert_eq!(res.max_supply, Uint128::new(10000000));
        assert_eq!(res.mint_price, Uint128::new(1000));
        assert_eq!(res.minting_enabled, true);
    }

    #[test]
    fn test_query_admin() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "uatom"));

        let msg = InstantiateMsg {
            name: "Polymera Token".to_string(),
            symbol: "POLY".to_string(),
            decimals: 6,
            initial_supply: Uint128::new(1000000),
            max_supply: Uint128::new(10000000),
            mint_price: Uint128::new(1000),
        };

        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let res = query_admin(deps.as_ref()).unwrap();
        assert_eq!(res, "creator");
    }
}
