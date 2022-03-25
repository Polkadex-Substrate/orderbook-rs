
extern crate orderbook;
extern crate rust_decimal;

use std::time::SystemTime;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use orderbook::{Orderbook, OrderSide, orders};


#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum BrokerAsset {
    USD,
    EUR,
    BTC,
    ETH,
}


fn parse_asset(asset: &str) -> Option<BrokerAsset> {
    match asset {
        "USD" => Some(BrokerAsset::USD),
        "EUR" => Some(BrokerAsset::EUR),
        "BTC" => Some(BrokerAsset::BTC),
        "ETH" => Some(BrokerAsset::ETH),
        _ => None,
    }
}


fn main() {
    let mut orderbook = Orderbook::new(BrokerAsset::BTC, BrokerAsset::USD);
    let order_asset = parse_asset("BTC").unwrap();
    let price_asset = parse_asset("USD").unwrap();

    // create order requests
    let order_list =
        vec![
            orders::new_limit_order_request(
                order_asset,
                price_asset,
                OrderSide::Bid,
                Decimal::from_f64(0.98).unwrap(),
                Decimal::from_f64(5.0).unwrap(),
                SystemTime::now()
            ),

            orders::new_limit_order_request(
                order_asset,
                price_asset,
                OrderSide::Ask,
                Decimal::from_f64(1.02).unwrap(),
                Decimal::from_f64(1.0).unwrap(),
                SystemTime::now()
            ),

            orders::amend_order_request(1, OrderSide::Bid, Decimal::from_f64(0.99).unwrap(),
                                        Decimal::from_f64(4.0).unwrap(), SystemTime::now()),

            orders::new_limit_order_request(
                order_asset,
                price_asset,
                OrderSide::Bid,
                Decimal::from_f64(1.01).unwrap(),
                Decimal::from_f64(0.4).unwrap(),
                SystemTime::now()
            ),

            orders::new_limit_order_request(
                order_asset,
                price_asset,
                OrderSide::Ask,
                Decimal::from_f64(1.03).unwrap(),
                Decimal::from_f64(0.5).unwrap(),
                SystemTime::now()
            ),

            orders::new_market_order_request(order_asset, price_asset, OrderSide::Bid, Decimal::from_f64(1.0).unwrap(), SystemTime::now()),

            orders::new_limit_order_request(
                order_asset,
                price_asset,
                OrderSide::Ask,
                Decimal::from_f64(1.05).unwrap(),
                Decimal::from_f64(0.5).unwrap(),
                SystemTime::now()
            ),

            orders::limit_order_cancel_request(4, OrderSide::Ask),

            orders::new_limit_order_request(
                order_asset,
                price_asset,
                OrderSide::Bid,
                Decimal::from_f64(1.06).unwrap(),
                Decimal::from_f64(0.6).unwrap(),
                SystemTime::now()
            ),
        ];

    // processing
    for order in order_list {
        println!("Order => {:?}", &order);
        let res = orderbook.process_order(order);
        println!("Processing => {:?}", res);
        if let Some((bid, ask)) = orderbook.current_spread() {
            println!("Spread => bid: {}, ask: {}\n", bid, ask);
        } else {
            println!("Spread => not available\n");
        }
    }
}
