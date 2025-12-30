use crate::{
    execution::{latency::LatencyModel, request::*},
    stream::hotloop::Mbo,
};

pub enum Ack {
    Accepted,
    Rejected,
}

pub enum Request {
    Trade(Trade),
    Modify(Modify),
    Cancel(Cancel),
}

impl Request {
    pub fn process<L: LatencyModel>(self, mbo: &Mbo, l: &L) {
        match self {
            Request::Trade(mut r) => r.submit(mbo, l),
            Request::Modify(mut r) => r.submit(mbo, l),
            Request::Cancel(mut r) => r.submit(mbo, l),
        }
    }
}
