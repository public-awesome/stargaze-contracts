use crate::admin::{
    can_execute, execute_freeze, execute_update_admins, query_admin_list, query_can_execute,
};
use crate::error::ContractError;
use crate::helpers::validators::map_validate;
use crate::msg::{
    ConfigResponse, ExecuteMsg, HasEndedResponse, HasMemberResponse,
    HasStartedResponse, InstantiateMsg, IsActiveResponse, QueryMsg,
};
use crate::state::{AdminList, Config, ADMIN_LIST, CONFIG, MERKLE_ROOT};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, StdResult, StdError, Timestamp};
use cw2::set_contract_version;
use sg_std::{Response, GENESIS_MINT_START_TIME, NATIVE_DENOM};
use cw_utils::nonpayable;


// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-whitelist-flex";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// contract governance params
pub const PRICE_PER_1000_MEMBERS: u128 = 100_000_000;
pub const MIN_MINT_PRICE: u128 = 0;


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    
    nonpayable(&info)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    MERKLE_ROOT.save(deps.storage, &msg.merkle_root)?;

    let config = Config {
        start_time: msg.start_time,
        end_time: msg.end_time,
        mint_price: msg.mint_price,
    };

    CONFIG.save(deps.storage, &config)?;

    let admin_config = AdminList {
        admins: map_validate(deps.api, &msg.admins)?,
        mutable: msg.admins_mutable,
    };
    ADMIN_LIST.save(deps.storage, &admin_config)?;

    if msg.start_time > msg.end_time {
        return Err(ContractError::InvalidStartTime(
            msg.start_time,
            msg.end_time,
        ));
    }

    if env.block.time >= msg.start_time {
        return Err(ContractError::InvalidStartTime(
            env.block.time,
            msg.start_time,
        ));
    }

    let genesis_start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    if msg.start_time < genesis_start_time {
        return Err(ContractError::InvalidStartTime(
            msg.start_time,
            genesis_start_time,
        ));
    }

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("sender", info.sender))
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateStartTime(time) => execute_update_start_time(deps, env, info, time),
        ExecuteMsg::UpdateEndTime(time) => execute_update_end_time(deps, env, info, time),
        ExecuteMsg::UpdateAdmins { admins } => execute_update_admins(deps, env, info, admins),
        ExecuteMsg::Freeze {} => execute_freeze(deps, env, info),
    }
}

pub fn execute_update_start_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_time: Timestamp,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    can_execute(&deps, info.sender.clone())?;

    // don't allow updating start time if whitelist is active
    if env.block.time >= config.start_time {
        return Err(ContractError::AlreadyStarted {});
    }

    if start_time > config.end_time {
        return Err(ContractError::InvalidStartTime(start_time, config.end_time));
    }

    let genesis_start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
    let start_time = if start_time < genesis_start_time {
        genesis_start_time
    } else {
        start_time
    };

    config.start_time = start_time;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_start_time")
        .add_attribute("start_time", start_time.to_string())
        .add_attribute("sender", info.sender))
}


pub fn execute_update_end_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    end_time: Timestamp,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    can_execute(&deps, info.sender.clone())?;

    if env.block.time >= config.start_time && end_time > config.end_time {
        return Err(ContractError::AlreadyStarted {});
    }

    if end_time < config.start_time {
        return Err(ContractError::InvalidEndTime(end_time, config.start_time));
    }

    config.end_time = end_time;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_end_time")
        .add_attribute("end_time", end_time.to_string())
        .add_attribute("sender", info.sender))
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::HasStarted {} => to_binary(&query_has_started(deps, env)?),
        QueryMsg::HasEnded {} => to_binary(&query_has_ended(deps, env)?),
        QueryMsg::IsActive {} => to_binary(&query_is_active(deps, env)?),
        QueryMsg::HasMember { member } => to_binary(&query_has_member(deps, member)?),
        QueryMsg::Config {} => to_binary(&query_config(deps, env)?),
        QueryMsg::AdminList {} => to_binary(&query_admin_list(deps)?),
        QueryMsg::CanExecute { sender, .. } => to_binary(&query_can_execute(deps, &sender)?),
    }
}

fn query_has_started(deps: Deps, env: Env) -> StdResult<HasStartedResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(HasStartedResponse {
        has_started: (env.block.time >= config.start_time),
    })
}

fn query_has_ended(deps: Deps, env: Env) -> StdResult<HasEndedResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(HasEndedResponse {
        has_ended: (env.block.time >= config.end_time),
    })
}

fn query_is_active(deps: Deps, env: Env) -> StdResult<IsActiveResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(IsActiveResponse {
        is_active: (env.block.time >= config.start_time) && (env.block.time < config.end_time),
    })
}


pub fn query_has_member(deps: Deps, member: String) -> StdResult<HasMemberResponse> {
    let _addr = deps.api.addr_validate(&member)?;
    Ok(HasMemberResponse {
        has_member: true // WHITELIST.has(deps.storage, addr),
    })
}

pub fn query_config(deps: Deps, env: Env) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        start_time: config.start_time,
        end_time: config.end_time,
        mint_price: config.mint_price,
        is_active: (env.block.time >= config.start_time) && (env.block.time < config.end_time),
    })
}

pub fn not_supported() -> StdResult<()> {
    Err(StdError::generic_err("Not supported"))
}
