pub mod model;

pub trait LatencyModel {
    fn time_delta(&self, ts_event: &u64) -> u64;
}
