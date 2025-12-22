use dbn::{Action, MboMsg, Side, UNDEF_PRICE};
use std::collections::{BTreeMap, HashMap, VecDeque};

#[derive(Debug, Default)]
pub struct Book {
    orders_by_id: HashMap<u64, (Side, i64)>,
    offers: BTreeMap<i64, Level>,
    bids: BTreeMap<i64, Level>,
}

type Level = VecDeque<MboMsg>;

impl Book {
    pub fn apply(&mut self, mbo: MboMsg) {
        let action = mbo.action().unwrap();
        match action {
            Action::Modify => self.modify(mbo),
            Action::Trade | Action::Fill | Action::None => {},
            Action::Cancel => self.cancel(mbo),
            Action::Add => self.add(mbo),
            Action::Clear => self.clear(),
        }
    }

    fn clear(&mut self) {
        self.orders_by_id.clear();
        self.offers.clear();
        self.bids.clear();
    }

    fn add(&mut self, mbo: MboMsg) {
        let price = mbo.price;
        let side = mbo.side().unwrap();
        if mbo.flags.is_tob() {
            let levels: &mut BTreeMap<i64, Level> = self.side_levels_mut(side);
            levels.clear();
            // UNDEF_PRICE indicates the side's book should be cleared
            // and doesn't represent an order that should be added
            if mbo.price != UNDEF_PRICE {
                levels.insert(price, VecDeque::from([mbo]));
            }
        } else {
            match self.orders_by_id.insert(mbo.order_id, (side, price)) {
                None => {
                    let level = self.get_or_insert_level(side, price);
                    level.push_back(mbo);
                },
                Some((old_side, old_price)) => {
                    self.remove_level(old_side, old_price);
                    let level = self.get_or_insert_level(side, price);
                    level.push_back(mbo);
                }
            }
        }
    }

    fn cancel(&mut self, mbo: MboMsg) {
        let side = mbo.side().unwrap();
        let level = self.level_mut(side, mbo.price);
        let order_idx = Self::find_order(level, mbo.order_id);
        let existing_order = level.get_mut(order_idx).unwrap();
        assert!(existing_order.size >= mbo.size);
        existing_order.size -= mbo.size;
        if existing_order.size == 0 {
            level.remove(order_idx).unwrap();
            if level.is_empty() {
                self.remove_level(side, mbo.price);
            }
            self.orders_by_id.remove(&mbo.order_id).unwrap();
        }
    }

    fn modify(&mut self, mbo: MboMsg) {
        let order_id = mbo.order_id;
        let side = mbo.side().unwrap();
        let Some((id_side, id_price)) = self.orders_by_id.get_mut(&order_id) else {
            // If order not found, treat it as an add
            return self.add(mbo);
        };
        let prev_side = *id_side;
        let prev_price = *id_price;
        // Update orders by ID
        *id_side = side;
        *id_price = mbo.price;
        // Update level order
        let level = self.level_mut(prev_side, prev_price);
        let order_idx = Self::find_order(level, order_id);
        let existing_order = level.get_mut(order_idx).unwrap();
        existing_order.size = mbo.size;
        let should_keep_priority = prev_price == mbo.price && existing_order.size >= mbo.size;
        if should_keep_priority {
            return;
        }
        if prev_price != mbo.price {
            let prev_level = level;
            Self::remove_order(prev_level, order_id);
            if prev_level.is_empty() {
                self.remove_level(side, prev_price);
            }
            let level = self.get_or_insert_level(side, mbo.price);
            level.push_back(mbo);
        } else {
            Self::remove_order(level, order_id);
            level.push_back(mbo);
        }
    }

    fn get_or_insert_level(&mut self, side: Side, price: i64) -> &mut Level {
        let levels = self.side_levels_mut(side);
        levels.entry(price).or_default()
    }

    fn level_mut(&mut self, side: Side, price: i64) -> &mut Level {
        let levels = self.side_levels_mut(side);
        levels.get_mut(&price).unwrap()
    }

    fn remove_level(&mut self, side: Side, price: i64) {
        self.side_levels_mut(side).remove(&price).unwrap();
    }

    fn find_order(level: &VecDeque<MboMsg>, order_id: u64) -> usize {
        level.iter().position(|o| o.order_id == order_id).unwrap()
    }

    fn remove_order(level: &mut VecDeque<MboMsg>, order_id: u64) {
        let index = Self::find_order(level, order_id);
        level.remove(index).unwrap();
    }

    fn side_levels_mut(&mut self, side: Side) -> &mut BTreeMap<i64, Level> {
        match side {
            Side::Ask => &mut self.offers,
            Side::Bid => &mut self.bids,
            Side::None => panic!("Invalid side None"),
        }
    }
}
