use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum BidOrAsk {
    Bid,
    Ask,
}

#[derive(Debug)]
pub struct OrderBook {
    bids: HashMap<Price, Limit>,
    asks: HashMap<Price, Limit>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            bids: HashMap::new(),
            asks: HashMap::new(),
        }
    }

    pub fn fill_market_order(&mut self, market_order: &mut Order) {
        match market_order.bid_or_ask {
            BidOrAsk::Bid => {
                for limit_order in self.ask_limits() {
                    limit_order.fill_order(market_order);
                    if market_order.is_filled() {
                        break;
                    }
                }
            }
            BidOrAsk::Ask => {
                for limit_order in self.bid_limits() {
                    limit_order.fill_order(market_order);
                    if market_order.is_filled() {
                        break;
                    }
                }
            }
        }
    }

    pub fn ask_limits(&mut self) -> Vec<&mut Limit> {
        let mut limits: Vec<&mut Limit> = self.asks.values_mut().collect::<Vec<&mut Limit>>();
        limits.sort_by(|a: &&mut Limit, b: &&mut Limit| a.price.partial_cmp(&b.price).unwrap());
        limits
    }

    pub fn bid_limits(&mut self) -> Vec<&mut Limit> {
        let mut limits: Vec<&mut Limit> = self.bids.values_mut().collect::<Vec<&mut Limit>>();
        limits.sort_by(|a: &&mut Limit, b: &&mut Limit| b.price.partial_cmp(&a.price).unwrap());
        limits
    }

    pub fn add_order(&mut self, price: f64, order: Order) {
        let price = Price::new(price);

        match order.bid_or_ask {
            BidOrAsk::Bid => match self.bids.get_mut(&price) {
                Some(limit) => {
                    limit.add_order(order);
                }
                None => {
                    let mut limit = Limit::new(price);
                    limit.add_order(order);
                    self.bids.insert(price, limit);
                }
            },
            BidOrAsk::Ask => match self.asks.get_mut(&price) {
                Some(limit) => {
                    limit.add_order(order);
                }
                None => {
                    let mut limit = Limit::new(price);
                    limit.add_order(order);
                    self.asks.insert(price, limit);
                }
            },
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Clone, Copy)]
pub struct Price {
    integral: u64,
    fractional: u64,
    scalar: u64,
}

impl Price {
    pub fn new(price: f64) -> Price {
        let scalar = 100000;
        let integral = price as u64;
        let fractional = (price.fract() * scalar as f64) as u64;
        Price {
            scalar,
            integral,
            fractional,
        }
    }
}

#[derive(Debug)]
pub struct Limit {
    price: Price,
    orders: Vec<Order>,
}

impl Limit {
    pub fn new(price: Price) -> Limit {
        Limit {
            price,
            orders: Vec::new(),
        }
    }

    pub fn total_volume(&self) -> f64 {
        self.orders.iter().map(|order| order.size).sum()
    }

    pub fn fill_order(&mut self, market_order: &mut Order) {
        for limit_order in self.orders.iter_mut() {
            if market_order.size >= limit_order.size {
                market_order.size -= limit_order.size;
                limit_order.size = 0.0;
            } else {
                limit_order.size -= market_order.size;
                market_order.size = 0.0;
            }

            if market_order.is_filled() {
                break;
            }
        }
    }

    pub fn add_order(&mut self, order: Order) {
        self.orders.push(order);
    }
}

#[derive(Debug)]
pub struct Order {
    size: f64,
    bid_or_ask: BidOrAsk,
}

impl Order {
    pub fn new(bid_or_ask: BidOrAsk, size: f64) -> Order {
        Order { size, bid_or_ask }
    }

    pub fn is_filled(&self) -> bool {
        self.size == 0.0
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_price_new() {
        let price = Price::new(1.23456789);
        assert_eq!(price.integral, 1);
        assert_eq!(price.fractional, 23456);
        assert_eq!(price.scalar, 100000);
    }

    #[test]
    fn test_limit_new() {
        let price = Price::new(1.23456789);
        let limit = Limit::new(price);
        assert_eq!(limit.price.integral, 1);
        assert_eq!(limit.price.fractional, 23456);
        assert_eq!(limit.price.scalar, 100000);
    }

    #[test]
    fn test_order_new() {
        let order = Order::new(BidOrAsk::Bid, 1.23456789);
        assert_eq!(order.size, 1.23456789);
        assert_eq!(order.bid_or_ask, BidOrAsk::Bid);
    }

    #[test]
    fn test_order_is_filled() {
        let mut order = Order::new(BidOrAsk::Bid, 1.23456789);
        assert_eq!(order.is_filled(), false);
        order.size = 0.0;
        assert_eq!(order.is_filled(), true);
    }

    #[test]
    fn test_orderbook_new() {
        let orderbook = OrderBook::new();
        assert_eq!(orderbook.bids.len(), 0);
        assert_eq!(orderbook.asks.len(), 0);
    }

    #[test]
    fn test_orderbook_add_order() {
        let mut orderbook = OrderBook::new();
        let order = Order::new(BidOrAsk::Bid, 1.23456789);
        orderbook.add_order(1.23456789, order);
        assert_eq!(orderbook.bids.len(), 1);
        assert_eq!(orderbook.asks.len(), 0);
        assert_eq!(
            orderbook
                .bids
                .get(&Price::new(1.23456789))
                .unwrap()
                .orders
                .len(),
            1
        );
        assert_eq!(
            orderbook.bids.get(&Price::new(1.23456789)).unwrap().orders[0].size,
            1.23456789
        );
        assert_eq!(
            orderbook.bids.get(&Price::new(1.23456789)).unwrap().orders[0].bid_or_ask,
            BidOrAsk::Bid
        );
    }

    #[test]
    fn test_limit_fill_order_with_single_limit() {
        let mut limit = Limit::new(Price::new(1.23456789));
        let buy_limit_order = Order::new(BidOrAsk::Bid, 100.0);
        limit.add_order(buy_limit_order);

        let mut market_sell_order = Order::new(BidOrAsk::Ask, 50.0);
        limit.fill_order(&mut market_sell_order);
        assert_eq!(market_sell_order.size, 0.0);
        assert_eq!(limit.orders[0].size, 50.0);
        assert_eq!(market_sell_order.is_filled(), true);

        let mut market_sell_order = Order::new(BidOrAsk::Ask, 100.0);
        limit.fill_order(&mut market_sell_order);
        assert_eq!(market_sell_order.size, 50.0);
        assert_eq!(limit.orders[0].size, 0.0);
        assert_eq!(market_sell_order.is_filled(), false);
    }

    #[test]
    fn test_limit_fill_order_with_multiple_limits() {
        let mut limit = Limit::new(Price::new(1.23456789));
        let buy_limit_order = Order::new(BidOrAsk::Bid, 50.0);
        limit.add_order(buy_limit_order);
        let buy_limit_order = Order::new(BidOrAsk::Bid, 50.0);
        limit.add_order(buy_limit_order);

        let mut market_sell_order = Order::new(BidOrAsk::Ask, 99.0);
        limit.fill_order(&mut market_sell_order);
        assert_eq!(market_sell_order.size, 0.0);
        assert_eq!(limit.orders[0].size, 0.0);
        assert_eq!(limit.orders[1].size, 1.0);
        assert_eq!(market_sell_order.is_filled(), true);
    }

    #[test]
    fn test_limit_total_volume() {
        let mut limit = Limit::new(Price::new(1.23456789));
        let buy_limit_order = Order::new(BidOrAsk::Bid, 50.0);
        limit.add_order(buy_limit_order);
        let buy_limit_order = Order::new(BidOrAsk::Bid, 50.0);
        limit.add_order(buy_limit_order);
        assert_eq!(limit.total_volume(), 100.0);
    }
}
