use crate::{execution::latency::LatencyModel, stream::hotloop::Mbo};

pub struct Trade {
    pub ts_event: u64,
    pub instrument_id: u32,
    pub side: i8,
    pub price: i64,
    pub size: u32,
    pub order_id: u64,
    pub time_delta: u64,
}

impl Trade {
    pub fn new(side: i8, price: i64, size: u32) -> Self {
        Self {
            ts_event: 0,
            instrument_id: 0,
            side,
            price,
            size,
            order_id: 0,
            time_delta: 0,
        }
    }

    pub fn submit<L: LatencyModel>(&mut self, mbo: &Mbo, latency: &L) {
        self.ts_event = mbo.ts_recv;
        self.time_delta = latency.time_delta(&self.ts_event);
        self.instrument_id = mbo.instrument_id;
    }
}

pub struct Modify {
    pub ts_event: u64,
    pub instrument_id: u32,
    pub price: Option<i64>,
    pub size: Option<u32>,
    pub order_id: u64,
    pub time_delta: u64,
}

impl Modify {
    pub fn new(price: Option<i64>, size: Option<u32>, order_id: u64) -> Self {
        Self {
            ts_event: 0,
            instrument_id: 0,
            price,
            size,
            order_id,
            time_delta: 0,
        }
    }

    pub fn submit<L: LatencyModel>(&mut self, mbo: &Mbo, latency: &L) {
        self.ts_event = mbo.ts_recv;
        self.time_delta = latency.time_delta(&self.ts_event);
        self.instrument_id = mbo.instrument_id;
    }
}

pub struct Cancel {
    pub ts_event: u64,
    pub instrument_id: u32,
    pub order_id: u64,
    pub time_delta: u64,
}

impl Cancel {
    pub fn new(order_id: u64) -> Self {
        Self {
            ts_event: 0,
            instrument_id: 0,
            order_id,
            time_delta: 0,
        }
    }

    pub fn submit<L: LatencyModel>(&mut self, mbo: &Mbo, latency: &L) {
        self.ts_event = mbo.ts_recv;
        self.time_delta = latency.time_delta(&self.ts_event);
        self.instrument_id = mbo.instrument_id;
    }
}
