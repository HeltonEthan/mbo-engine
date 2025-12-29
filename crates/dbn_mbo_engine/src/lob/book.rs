use std::collections::{BTreeMap, VecDeque};
use dbn::{FlagSet, UNDEF_PRICE};
use hashbrown::HashMap;

use crate::stream::hotloop::Mbo;

#[derive(Debug, Default)]
pub struct Book {
    /// order_id -> (side, price)
    pub orders_by_id: HashMap<u64, (i8, i64)>,
    pub offers: BTreeMap<i64, Level>,
    pub bids: BTreeMap<i64, Level>,
}

type Level = VecDeque<LobMbo>;

impl Book {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply(&mut self, mbo: LobMbo) {
        match mbo.action {
            val if val == b'M' as i8 => self.modify(mbo),
            val if val == b'C' as i8 => self.cancel(mbo),
            val if val == b'A' as i8 => self.add(mbo),
            val if val == b'R' as i8 => self.clear(),
            _ => {}
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.orders_by_id.clear();
        self.offers.clear();
        self.bids.clear();
    }

    #[inline]
    pub fn add(&mut self, mbo: LobMbo) {
        let price = mbo.price;
        let side = mbo.side;
        if mbo.flags.is_tob() {
            let levels = self.side_levels_mut(side);
            levels.clear();
            // UNDEF_PRICE indicates the side's book should be cleared
            // and doesn't represent an order that should be added
            if price != UNDEF_PRICE {
                levels.insert(price, VecDeque::from())
            }
        }
    }

    #[inline]
    fn side_levels_mut(&mut self, side: i8) -> &mut BTreeMap<i64, Level> {
        match side {
            val if val == b'A' as i8 => &mut self.offers,
            val if val == b'B' as i8 => &mut self.bids,
            _ => panic!("Invalid side None"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LobMbo {
    pub ts_recv: u64,
    pub ts_event: u64,
    pub action: i8,
    pub side: i8,
    pub price: i64,
    pub size: u32,
    pub order_id: u64,
    pub flags: FlagSet,
}

impl From<&Mbo> for LobMbo {
    #[inline]
    fn from(mbo: &Mbo) -> Self {
        Self {
            ts_recv: mbo.ts_recv,
            ts_event: mbo.ts_event,
            action: mbo.action,
            side: mbo.side,
            price: mbo.price,
            size: mbo.size,
            order_id: mbo.order_id,
            flags: mbo.flags,
        }
    }
}
