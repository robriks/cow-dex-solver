//! Module containing implementation of the 0x solver.
//!
//! This solver will simply use the 0x API to get a quote for a
//! single GPv2 order and produce a settlement directly against 0x.
//!
//! Please be aware of the following subtlety for buy orders:
//! 0x's API is adding the defined slippage on the sellAmount of a buy order
//! and then returns the surplus in the buy amount to the user.
//! I.e. if the user defines a 5% slippage, they will sell 5% more, and receive 5%
//! more buy-tokens than ordered. Here is on example tx:
//! https://dashboard.tenderly.co/gp-v2/staging/simulator/new?block=12735030&blockIndex=0&from=0xa6ddbd0de6b310819b49f680f65871bee85f517e&gas=8000000&gasPrice=0&value=0&contractAddress=0x3328f5f2cecaf00a2443082b657cedeaf70bfaef&rawFunctionInput=0x13d79a0b000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000003600000000000000000000000000000000000000000000000000000000000000002000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000dac17f958d2ee523a2206206994597c13d831ec7000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000029143e200000000000000000000000000000000000000000000000000470de4df820000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000036416d81e590ff67370e4523b9cd3257aa0a853c000000000000000000000000000000000000000000000000000000000291f64800000000000000000000000000000000000000000000000000470de4df8200000000000000000000000000000000000000000000000000000000000060dc5839000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000003dc140000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000029143e2000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000000410a7f27a6638cc9cdaba8266a15acef4cf7e1e1c9b9b2059391b7230b67bdfeb21f1d3aa45852f527a5040d3d7a190b92764a2c854f334b7eed579b390b85fd3f1b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000003800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000120000000000000000000000000dac17f958d2ee523a2206206994597c13d831ec7000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000044095ea7b3000000000000000000000000def1c0ded9bec7f1a1670819833240f027b25effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00000000000000000000000000000000000000000000000000000000000000000000000000000000def1c0ded9bec7f1a1670819833240f027b25eff000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000128d9627aa400000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000002b220e100000000000000000000000000000000000000000000000000470de4df82000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000dac17f958d2ee523a2206206994597c13d831ec7000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2869584cd000000000000000000000000100000000000000000000000000000000000001100000000000000000000000000000000000000000000003239e38b8a60dc53b70000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000&network=1
//! This behavior has the following risks: The additional sell tokens from the slippage
//! are not provided by the user, hence the additional tokens might be not available in
//! the settlement contract. For smaller amounts this is unlikely, as we always charge the
//! fees also in the sell token, though, the fee's might not always be sufficient.
//! This risk should be covered in a future PR.
//!
//! Sell orders are unproblematic, especially, since the positive slippage is handed back from 0x

pub mod api;

use crate::solve::zeroex_solver::api::ZeroExApi;
use anyhow::{ensure, Result};
use reqwest::Client;

use self::api::DefaultZeroExApi;

use std::fmt::{self, Display, Formatter};

// A GPv2 solver that matches GP orders to direct 0x swaps.
pub struct ZeroExSolver {
    pub client: Box<dyn ZeroExApi + Send + Sync>,
}

/// Chain ID for Mainnet.
const MAINNET_CHAIN_ID: u64 = 1;

impl ZeroExSolver {
    pub fn new(chain_id: u64, client: Client) -> Result<Self> {
        ensure!(
            chain_id == MAINNET_CHAIN_ID,
            "0x solver only supported on Mainnet",
        );
        Ok(Self {
            client: Box::new(DefaultZeroExApi::new(
                DefaultZeroExApi::DEFAULT_URL,
                client,
            )?),
        })
    }
}

// fn swap_respects_limit_price(swap: &SwapResponse, order: &OrderModel) -> bool {
//     match order.is_sell_order {
//         false => swap.sell_amount <= order.sell_amount,
//         true => swap.buy_amount >= order.buy_amount,
//     }
// }

impl Display for ZeroExSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ZeroExSolver")
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::interactions::allowances::{Approval, MockAllowanceManaging};
//     use crate::liquidity::tests::CapturingSettlementHandler;
//     use crate::liquidity::LimitOrder;
//     use crate::solver::zeroex_solver::api::MockZeroExApi;
//     use crate::test::account;
//     use contracts::{GPv2Settlement, WETH9};
//     use ethcontract::{Web3, H160, U256};
//     use mockall::predicate::*;
//     use mockall::Sequence;
//     use model::order::{Order, OrderCreation, OrderKind};
//     use shared::transport::{create_env_test_transport, create_test_transport};

//     #[tokio::test]
//     #[ignore]
//     async fn solve_sell_order_on_zeroex() {
//         let web3 = Web3::new(create_env_test_transport());
//         let chain_id = web3.eth().chain_id().await.unwrap().as_u64();
//         let settlement = GPv2Settlement::deployed(&web3).await.unwrap();

//         let weth = WETH9::deployed(&web3).await.unwrap();
//         let gno = shared::addr!("6810e776880c02933d47db1b9fc05908e5386b96");

//         let solver =
//             ZeroExSolver::new(account(), web3, settlement, chain_id, Client::new()).unwrap();
//         let settlement = solver
//             .try_settle_order(
//                 Order {
//                     order_creation: OrderCreation {
//                         sell_token: weth.address(),
//                         buy_token: gno,
//                         sell_amount: 1_000_000_000_000_000_000u128.into(),
//                         buy_amount: 2u128.into(),
//                         kind: OrderKind::Sell,
//                         ..Default::default()
//                     },
//                     ..Default::default()
//                 }
//                 .into(),
//             )
//             .await
//             .unwrap();

//         println!("{:#?}", settlement);
//     }

//     #[tokio::test]
//     #[ignore]
//     async fn solve_buy_order_on_zeroex() {
//         let web3 = Web3::new(create_env_test_transport());
//         let chain_id = web3.eth().chain_id().await.unwrap().as_u64();
//         let settlement = GPv2Settlement::deployed(&web3).await.unwrap();

//         let weth = WETH9::deployed(&web3).await.unwrap();
//         let gno = shared::addr!("6810e776880c02933d47db1b9fc05908e5386b96");

//         let solver =
//             ZeroExSolver::new(account(), web3, settlement, chain_id, Client::new()).unwrap();
//         let settlement = solver
//             .try_settle_order(
//                 Order {
//                     order_creation: OrderCreation {
//                         sell_token: weth.address(),
//                         buy_token: gno,
//                         sell_amount: 1_000_000_000_000_000_000u128.into(),
//                         buy_amount: 1_000_000_000_000_000_000u128.into(),
//                         kind: OrderKind::Buy,
//                         ..Default::default()
//                     },
//                     ..Default::default()
//                 }
//                 .into(),
//             )
//             .await
//             .unwrap();

//         println!("{:#?}", settlement);
//     }

//     #[tokio::test]
//     async fn test_satisfies_limit_price_for_orders() {
//         let mut client = Box::new(MockZeroExApi::new());
//         let mut allowance_fetcher = Box::new(MockAllowanceManaging::new());

//         let sell_token = H160::from_low_u64_be(1);
//         let buy_token = H160::from_low_u64_be(1);

//         let allowance_target = shared::addr!("def1c0ded9bec7f1a1670819833240f027b25eff");
//         client.expect_get_swap().returning(move |_| {
//             Ok(SwapResponse {
//                 sell_amount: U256::from_dec_str("100").unwrap(),
//                 buy_amount: U256::from_dec_str("91").unwrap(),
//                 allowance_target,
//                 price: 0.91_f64,
//                 to: shared::addr!("0000000000000000000000000000000000000000"),
//                 data: web3::types::Bytes(hex::decode("00").unwrap()),
//                 value: U256::from_dec_str("0").unwrap(),
//             })
//         });

//         allowance_fetcher
//             .expect_get_approval()
//             .times(2)
//             .with(eq(sell_token), eq(allowance_target), eq(U256::from(100)))
//             .returning(move |_, _, _| {
//                 Ok(Approval::Approve {
//                     token: sell_token,
//                     spender: allowance_target,
//                 })
//             });

//         let solver = ZeroExSolver {
//             account: account(),
//             client,
//             allowance_fetcher,
//         };

//         let buy_order_passing_limit = LimitOrder {
//             sell_token,
//             buy_token,
//             sell_amount: 101.into(),
//             buy_amount: 91.into(),
//             kind: model::order::OrderKind::Buy,
//             ..Default::default()
//         };
//         let buy_order_violating_limit = LimitOrder {
//             sell_token,
//             buy_token,
//             sell_amount: 99.into(),
//             buy_amount: 91.into(),
//             kind: model::order::OrderKind::Buy,
//             ..Default::default()
//         };
//         let sell_order_passing_limit = LimitOrder {
//             sell_token,
//             buy_token,
//             sell_amount: 100.into(),
//             buy_amount: 90.into(),
//             kind: model::order::OrderKind::Sell,
//             ..Default::default()
//         };
//         let sell_order_violating_limit = LimitOrder {
//             sell_token,
//             buy_token,
//             sell_amount: 100.into(),
//             buy_amount: 110.into(),
//             kind: model::order::OrderKind::Sell,
//             ..Default::default()
//         };

//         let result = solver
//             .try_settle_order(sell_order_passing_limit)
//             .await
//             .unwrap()
//             .unwrap();
//         assert_eq!(
//             result.clearing_prices(),
//             &hashmap! {
//                 sell_token => 91.into(),
//                 buy_token => 100.into(),
//             }
//         );

//         let result = solver
//             .try_settle_order(sell_order_violating_limit)
//             .await
//             .unwrap();
//         assert!(result.is_none());

//         let result = solver
//             .try_settle_order(buy_order_passing_limit)
//             .await
//             .unwrap()
//             .unwrap();
//         assert_eq!(
//             result.clearing_prices(),
//             &hashmap! {
//                 sell_token => 91.into(),
//                 buy_token => 100.into(),
//             }
//         );

//         let result = solver
//             .try_settle_order(buy_order_violating_limit)
//             .await
//             .unwrap();
//         assert!(result.is_none());
//     }

//     #[tokio::test]
//     #[ignore]
//     async fn returns_error_on_non_mainnet() {
//         let web3 = Web3::new(create_test_transport(
//             &std::env::var("NODE_URL_RINKEBY").unwrap(),
//         ));
//         let chain_id = web3.eth().chain_id().await.unwrap().as_u64();
//         let settlement = GPv2Settlement::deployed(&web3).await.unwrap();

//         assert!(ZeroExSolver::new(account(), web3, settlement, chain_id, Client::new()).is_err())
//     }

//     #[tokio::test]
//     async fn test_sets_allowance_if_necessary() {
//         let mut client = Box::new(MockZeroExApi::new());
//         let mut allowance_fetcher = Box::new(MockAllowanceManaging::new());

//         let sell_token = H160::from_low_u64_be(1);
//         let buy_token = H160::from_low_u64_be(1);

//         let allowance_target = shared::addr!("def1c0ded9bec7f1a1670819833240f027b25eff");
//         client.expect_get_swap().returning(move |_| {
//             Ok(SwapResponse {
//                 sell_amount: U256::from_dec_str("100").unwrap(),
//                 buy_amount: U256::from_dec_str("91").unwrap(),
//                 allowance_target,
//                 price: 13.121_002_575_170_278_f64,
//                 to: shared::addr!("0000000000000000000000000000000000000000"),
//                 data: web3::types::Bytes(hex::decode("").unwrap()),
//                 value: U256::from_dec_str("0").unwrap(),
//             })
//         });

//         // On first invocation no prior allowance, then max allowance set.
//         let mut seq = Sequence::new();
//         allowance_fetcher
//             .expect_get_approval()
//             .times(1)
//             .with(eq(sell_token), eq(allowance_target), eq(U256::from(100)))
//             .returning(move |_, _, _| {
//                 Ok(Approval::Approve {
//                     token: sell_token,
//                     spender: allowance_target,
//                 })
//             })
//             .in_sequence(&mut seq);
//         allowance_fetcher
//             .expect_get_approval()
//             .times(1)
//             .returning(|_, _, _| Ok(Approval::AllowanceSufficient))
//             .in_sequence(&mut seq);

//         let solver = ZeroExSolver {
//             account: account(),
//             client,
//             allowance_fetcher,
//         };

//         let order = LimitOrder {
//             sell_token,
//             buy_token,
//             sell_amount: 100.into(),
//             buy_amount: 90.into(),
//             ..Default::default()
//         };

//         // On first run we have two main interactions (approve + swap)
//         let result = solver
//             .try_settle_order(order.clone())
//             .await
//             .unwrap()
//             .unwrap();
//         assert_eq!(result.encoder.finish().interactions[1].len(), 2);

//         // On second run we have only have one main interactions (swap)
//         let result = solver.try_settle_order(order).await.unwrap().unwrap();
//         assert_eq!(result.encoder.finish().interactions[1].len(), 1)
//     }

//     #[tokio::test]
//     async fn sets_execution_amount_based_on_kind() {
//         let sell_token = H160::from_low_u64_be(1);
//         let buy_token = H160::from_low_u64_be(2);

//         let mut client = Box::new(MockZeroExApi::new());
//         client.expect_get_swap().returning(move |_| {
//             Ok(SwapResponse {
//                 sell_amount: 1000.into(),
//                 buy_amount: 5000.into(),
//                 allowance_target: shared::addr!("0000000000000000000000000000000000000000"),
//                 price: 0.,
//                 to: shared::addr!("0000000000000000000000000000000000000000"),
//                 data: web3::types::Bytes(vec![]),
//                 value: 0.into(),
//             })
//         });

//         let mut allowance_fetcher = Box::new(MockAllowanceManaging::new());
//         allowance_fetcher
//             .expect_get_approval()
//             .returning(|_, _, _| Ok(Approval::AllowanceSufficient));

//         let solver = ZeroExSolver {
//             account: account(),
//             client,
//             allowance_fetcher,
//         };

//         let order = LimitOrder {
//             sell_token,
//             buy_token,
//             sell_amount: 1234.into(),
//             buy_amount: 4321.into(),
//             ..Default::default()
//         };

//         // Sell orders are fully executed
//         let handler = CapturingSettlementHandler::arc();
//         solver
//             .try_settle_order(LimitOrder {
//                 kind: OrderKind::Sell,
//                 settlement_handling: handler.clone(),
//                 ..order.clone()
//             })
//             .await
//             .unwrap()
//             .unwrap();
//         assert_eq!(handler.calls(), vec![1234.into()]);

//         // Buy orders are fully executed
//         let handler = CapturingSettlementHandler::arc();
//         solver
//             .try_settle_order(LimitOrder {
//                 kind: OrderKind::Buy,
//                 settlement_handling: handler.clone(),
//                 ..order
//             })
//             .await
//             .unwrap()
//             .unwrap();
//         assert_eq!(handler.calls(), vec![4321.into()]);
//     }
// }
