mod engine;

pub use engine::domain::OrderSide;
pub use engine::orderbook::{Orderbook, OrderProcessingResult, Success, Failed};
pub use engine::orders;


#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use rust_decimal::prelude::FromPrimitive;
    use super::*;


    fn match_float(expected: Decimal, get: Decimal) -> bool {
        if (expected - get).abs() < Decimal::from_f64(1e-6).unwrap() {
            true
        } else {
            false
        }
    }

    #[derive(PartialEq, Eq, Debug, Copy, Clone)]
    enum Asset {
        USD,
        EUR,
        BTC,
        ETH,
    }

    fn parse_asset(asset: &str) -> Option<Asset> {
        match asset {
            "USD" => Some(Asset::USD),
            "EUR" => Some(Asset::EUR),
            "BTC" => Some(Asset::BTC),
            "ETH" => Some(Asset::ETH),
            _ => None,
        }
    }

    #[test]
    fn market_order_on_empty_orderbook() {
        use std::time::SystemTime;

        let mut orderbook = Orderbook::new(Asset::BTC, Asset::USD);
        let order_asset = parse_asset("BTC").unwrap();
        let price_asset = parse_asset("USD").unwrap();

        let order1 = orders::new_market_order_request(
            order_asset,
            price_asset,
            OrderSide::Bid,
            Decimal::from_f64(2.0).unwrap(),
            SystemTime::now(),
        );

        // process market order
        let res = orderbook.process_order(order1);

        if !match res[0] {
            Ok(Success::Accepted { id: 1, .. }) => true,
            _ => false,
        } ||
            !match res[1] {
                Err(Failed::NoMatch(1)) => true,
                _ => false,
            }
        {
            panic!("unexpected event sequence: {:?}", res)
        }
    }


    #[test]
    fn market_order_partial_match() {
        use std::time::SystemTime;

        let mut orderbook = Orderbook::new(Asset::BTC, Asset::USD);
        let order_asset = parse_asset("BTC").unwrap();
        let price_asset = parse_asset("USD").unwrap();

        let order1 = orders::new_limit_order_request(
            order_asset,
            price_asset,
            OrderSide::Bid,
            Decimal::from_f64(10.0).unwrap(),
            Decimal::from_f64(1.0).unwrap(),
            SystemTime::now(),
        );

        let order2 = orders::new_market_order_request(
            order_asset,
            price_asset,
            OrderSide::Ask,
            Decimal::from_f64(0.5).unwrap(),
            SystemTime::now(),
        );

        orderbook.process_order(order1);
        let res = orderbook.process_order(order2);

        if !match res[0] {
            Ok(Success::Accepted { id: 2, .. }) => true,
            _ => false,
        } ||
            !match res[1] {
                Ok(Success::Filled {
                       order_id: 2,
                       price,
                       qty,
                       ..
                   }) if match_float(price, Decimal::from_f64(10.0).unwrap()) && match_float(qty, Decimal::from_f64(0.5).unwrap()) => true,
                _ => false,
            } ||
            !match res[2] {
                Ok(Success::PartiallyFilled {
                       order_id: 1,
                       price,
                       qty,
                       ..
                   }) if match_float(price, Decimal::from_f64(10.0).unwrap()) && match_float(qty, Decimal::from_f64(0.5).unwrap()) => true,
                _ => false,
            }
        {
            panic!("unexpected event sequence: {:?}", res)
        }
    }


    #[test]
    fn market_order_two_orders_match() {
        use std::time::SystemTime;

        let mut orderbook = Orderbook::new(Asset::BTC, Asset::USD);
        let order_asset = parse_asset("BTC").unwrap();
        let price_asset = parse_asset("USD").unwrap();

        let order1 = orders::new_limit_order_request(
            order_asset,
            price_asset,
            OrderSide::Bid,
            Decimal::from_f64(10.0).unwrap(),
            Decimal::from_f64(1.0).unwrap(),
            SystemTime::now(),
        );

        let order2 = orders::new_limit_order_request(
            order_asset,
            price_asset,
            OrderSide::Bid,
            Decimal::from_f64(12.0).unwrap(),
            Decimal::from_f64(1.0).unwrap(),
            SystemTime::now(),
        );

        let order3 = orders::new_market_order_request(
            order_asset,
            price_asset,
            OrderSide::Ask,
            Decimal::from_f64(1.5).unwrap(),
            SystemTime::now(),
        );

        orderbook.process_order(order1);
        orderbook.process_order(order2);
        let res = orderbook.process_order(order3);

        if !match res[0] {
            Ok(Success::Accepted { id: 3, .. }) => true,
            _ => false,
        } ||
            !match res[1] {
                Ok(Success::PartiallyFilled {
                       order_id: 3,
                       price,
                       qty,
                       ..
                   }) if match_float(price, Decimal::from_f64(12.0).unwrap()) && match_float(qty, Decimal::from_f64(1.0).unwrap()) => true,
                _ => false,
            } ||
            !match res[2] {
                Ok(Success::Filled {
                       order_id: 2,
                       price,
                       qty,
                       ..
                   }) if match_float(price, Decimal::from_f64(12.0).unwrap()) && match_float(qty, Decimal::from_f64(1.0).unwrap()) => true,
                _ => false,
            } ||
            !match res[3] {
                Ok(Success::Filled {
                       order_id: 3,
                       price,
                       qty,
                       ..
                   }) if match_float(price, Decimal::from_f64(10.0).unwrap()) && match_float(qty, Decimal::from_f64(0.5).unwrap()) => true,
                _ => false,
            } ||
            !match res[4] {
                Ok(Success::PartiallyFilled {
                       order_id: 1,
                       price,
                       qty,
                       ..
                   }) if match_float(price, Decimal::from_f64(10.0).unwrap()) && match_float(qty, Decimal::from_f64(0.5).unwrap()) => true,
                _ => false,
            }
        {
            panic!("unexpected event sequence: {:?}", res)
        }
    }


    #[test]
    fn limit_order_on_empty_orderbook() {
        use std::time::SystemTime;

        let mut orderbook = Orderbook::new(Asset::BTC, Asset::USD);
        let order_asset = parse_asset("BTC").unwrap();
        let price_asset = parse_asset("USD").unwrap();

        let order1 = orders::new_limit_order_request(
            order_asset,
            price_asset,
            OrderSide::Bid,
            Decimal::from_f64(10.0).unwrap(),
            Decimal::from_f64(2.0).unwrap(),
            SystemTime::now(),
        );

        // process order
        let res = orderbook.process_order(order1);

        if !match res[0] {
            Ok(Success::Accepted { id: 1, .. }) => true,
            _ => false,
        }
        {
            panic!("unexpected event sequence: {:?}", res)
        }
    }


    #[test]
    fn limit_order_partial_match() {
        use std::time::SystemTime;

        let mut orderbook = Orderbook::new(Asset::BTC, Asset::USD);
        let order_asset = parse_asset("BTC").unwrap();
        let price_asset = parse_asset("USD").unwrap();

        let order1 = orders::new_limit_order_request(
            order_asset,
            price_asset,
            OrderSide::Bid,
            Decimal::from_f64(10.0).unwrap(),
            Decimal::from_f64(1.0).unwrap(),
            SystemTime::now(),
        );

        let order2 = orders::new_limit_order_request(
            order_asset,
            price_asset,
            OrderSide::Ask,
            Decimal::from_f64(9.0).unwrap(),
            Decimal::from_f64(0.5).unwrap(),
            SystemTime::now(),
        );

        orderbook.process_order(order1);
        let res = orderbook.process_order(order2);

        if !match res[0] {
            Ok(Success::Accepted { id: 2, .. }) => true,
            _ => false,
        } ||
            !match res[1] {
                Ok(Success::Filled {
                       order_id: 2,
                       price,
                       qty,
                       ..
                   }) if match_float(price, Decimal::from_f64(10.0).unwrap()) && match_float(qty, Decimal::from_f64(0.5).unwrap()) => true,
                _ => false,
            } ||
            !match res[2] {
                Ok(Success::PartiallyFilled {
                       order_id: 1,
                       price,
                       qty,
                       ..
                   }) if match_float(price, Decimal::from_f64(10.0).unwrap()) && match_float(qty, Decimal::from_f64(0.5).unwrap()) => true,
                _ => false,
            }
        {
            panic!("unexpected event sequence: {:?}", res)
        }
    }


    #[test]
    fn limit_order_exact_match() {
        use std::time::SystemTime;

        let mut orderbook = Orderbook::new(Asset::BTC, Asset::USD);
        let order_asset = parse_asset("BTC").unwrap();
        let price_asset = parse_asset("USD").unwrap();

        let order1 = orders::new_limit_order_request(
            order_asset,
            price_asset,
            OrderSide::Bid,
            Decimal::from_f64(10.0).unwrap(),
            Decimal::from_f64(1.0).unwrap(),
            SystemTime::now(),
        );

        let order2 = orders::new_limit_order_request(
            order_asset,
            price_asset,
            OrderSide::Ask,
            Decimal::from_f64(9.0).unwrap(),
            Decimal::from_f64(0.5).unwrap(),
            SystemTime::now(),
        );

        orderbook.process_order(order1);
        let res = orderbook.process_order(order2);

        if !match res[0] {
            Ok(Success::Accepted { id: 2, .. }) => true,
            _ => false,
        } ||
            !match res[1] {
                Ok(Success::Filled {
                       order_id: 2,
                       price,
                       qty,
                       ..
                   }) if match_float(price, Decimal::from_f64(10.0).unwrap()) && match_float(qty, Decimal::from_f64(0.5).unwrap()) => true,
                _ => false,
            } ||
            !match res[2] {
                Ok(Success::PartiallyFilled {
                       order_id: 1,
                       price,
                       qty,
                       ..
                   }) if match_float(price, Decimal::from_f64(10.0).unwrap()) && match_float(qty, Decimal::from_f64(0.5).unwrap()) => true,
                _ => false,
            }
        {
            panic!("unexpected event sequence: {:?}", res)
        }

        let order3 = orders::new_limit_order_request(
            order_asset,
            price_asset,
            OrderSide::Ask,
            Decimal::from_f64(8.0).unwrap(),
            Decimal::from_f64(0.5).unwrap(),
            SystemTime::now(),
        );

        let res2 = orderbook.process_order(order3);

        if !match res2[0] {
            Ok(Success::Accepted { id: 3, .. }) => true,
            _ => false,
        } ||
            !match res2[1] {
                Ok(Success::Filled {
                       order_id: 3,
                       price,
                       qty,
                       ..
                   }) if match_float(price, Decimal::from_f64(10.0).unwrap()) && match_float(qty, Decimal::from_f64(0.5).unwrap()) => true,
                _ => false,
            } ||
            !match res2[2] {
                Ok(Success::Filled {
                       order_id: 1,
                       price,
                       qty,
                       ..
                   }) if match_float(price, Decimal::from_f64(10.0).unwrap()) && match_float(qty, Decimal::from_f64(0.5).unwrap()) => true,
                _ => false,
            }
        {
            panic!("unexpected event sequence: {:?}", res2)
        }

        assert_eq!(orderbook.current_spread(), None);
    }


    #[test]
    fn current_spread() {
        use std::time::SystemTime;

        let mut orderbook = Orderbook::new(Asset::BTC, Asset::USD);
        let order_asset = parse_asset("BTC").unwrap();
        let price_asset = parse_asset("USD").unwrap();

        let order1 = orders::new_limit_order_request(
            order_asset,
            price_asset,
            OrderSide::Bid,
            Decimal::from_f64(10.0).unwrap(),
            Decimal::from_f64(1.0).unwrap(),
            SystemTime::now(),
        );

        // not enough orders to calculate
        assert_eq!(orderbook.current_spread(), None);

        let order2 = orders::new_limit_order_request(
            order_asset,
            price_asset,
            OrderSide::Ask,
            Decimal::from_f64(12.0).unwrap(),
            Decimal::from_f64(0.5).unwrap(),
            SystemTime::now(),
        );

        let order3 = orders::new_limit_order_request(
            order_asset,
            price_asset,
            OrderSide::Ask,
            Decimal::from_f64(12.5).unwrap(),
            Decimal::from_f64(2.5).unwrap(),
            SystemTime::now(),
        );

        orderbook.process_order(order1);
        orderbook.process_order(order2);
        orderbook.process_order(order3);

        assert_eq!(orderbook.current_spread(), Some((Decimal::from_f64(10.0).unwrap(), Decimal::from_f64(12.0).unwrap())));

        // wider spread
        let order4 = orders::new_limit_order_request(
            order_asset,
            price_asset,
            OrderSide::Bid,
            Decimal::from_f64(14.0).unwrap(),
            Decimal::from_f64(1.5).unwrap(),
            SystemTime::now(),
        );
        let res = orderbook.process_order(order4);
        println!("{:?}", res);

        assert_eq!(orderbook.current_spread(), Some((Decimal::from_f64(10.0).unwrap(), Decimal::from_f64(12.5).unwrap())));
    }
}
