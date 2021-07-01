use cosmwasm_std::{Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, QueryRequest, StdError, StdResult, Storage, Uint128, WasmQuery, log, to_binary};
use terraswap::querier::query_balance;
use terraswap::asset::{Asset, AssetInfo};
use crate::msg::{LastWinnerResponse, HandleMsg, InitMsg, QueryMsg, TerrandQueryMsg, LatestRandomResponse};
use crate::state::{Config, LastWinner, config, config_read, last_winner, last_winner_read};

/////////////////////
/// Init entry point
/////////////////////
pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = Config {
        owner: deps.api.canonical_address(&env.message.sender)?,
        terrand_address: deps.api.canonical_address(&msg.terrand_address)?,
        minimum_bet_amount: msg.minimum_bet_amount,
    };

    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

/////////////////////
/// Handle entry point
/////////////////////
pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::GuessNumber {
            number,
        } => try_guess_number(deps, env, number),
    }
}

pub fn try_guess_number<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    number: u8,
) -> StdResult<HandleResponse> {
    let config: Config = config_read(&deps.storage).load()?;

    // check how much the user sent
    let sent_uusd: Uint128 = get_sent_uusd(&env)?;
    if sent_uusd < config.minimum_bet_amount {
        return Err(StdError::generic_err("Sent amount is less than minimum"));
    }

    // get randomess from a randomness oracle
    let res: LatestRandomResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: deps.api.human_address(&config.terrand_address)?,
        msg: to_binary(&TerrandQueryMsg::LatestDrand {})?,
    }))?;

    // calcualte the winner number
    let winner_number: u8 = res.randomness.as_slice()[1];

    // if the number is wrong, directly return
    if number != winner_number {
        return Ok(HandleResponse::default())
    }

    // calculate pot amount and send it to the winner
    let contract_ust_balance: Uint128 = query_balance(&deps, &env.contract.address, "uusd".to_string())?;
    let prize = Asset {
        info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        amount: contract_ust_balance,
    };

    // save last winner
    last_winner(&mut deps.storage).save(&LastWinner{
        address: deps.api.canonical_address(&env.message.sender)?,
        pot_amount: contract_ust_balance,
    })?;
    
    Ok(HandleResponse{
        messages: vec![
            prize.into_msg(&deps, env.contract.address, env.message.sender)?
        ],
        log: vec![
            log("action", "guess_number"),
        ],
        data: None,
    })
}

/////////////////////
/// Query entry point
/////////////////////
pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::LastWinner {} => to_binary(&query_last_winner(deps)?),
    }
}

fn query_last_winner<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<LastWinnerResponse> {
    let last_winner = last_winner_read(&deps.storage).load()?;
    Ok(LastWinnerResponse { 
        address: deps.api.human_address(&last_winner.address)?,
        pot_amount: last_winner.pot_amount,
    })
}

/////////////////////
/// Helper functions
/////////////////////
fn get_sent_uusd(env: &Env) -> StdResult<Uint128> {
    let sent_amount = Uint128::from(
        env.message
            .sent_funds
            .iter()
            .find(|c| c.denom == "uusd")
            .map(|c| c.amount)
            .ok_or_else(|| {
                StdError::generic_err("No uusd has been sent")
            })?,
    );
    Ok(sent_amount)
}
