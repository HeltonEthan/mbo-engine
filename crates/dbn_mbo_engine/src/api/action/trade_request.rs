use super::*;
use dbn::{Action, Side};

use crate::api_internal::submit::Order;

#[derive(Debug)]
pub struct TradeRequest {
    pub instrument_id: u32,
    pub side: Side,
    pub price: i64,
    pub size: u32,
}

impl TradeRequest {}

impl Submit for TradeRequest {
    fn submit<LM: LatencyModel>(&self, mbo: &MboMsg, latency: &mut LM) -> Ack {
        let ts_event = mbo.ts_recv;
        let ts_recv = latency.ts_recv_sim(ts_event);
        let order = Order::new(
            ts_recv,
            ts_event,
            self.instrument_id,
            Action::Trade,
            self.side,
            Some(self.price),
            Some(self.size),
        );
        todo!()
    }
}
