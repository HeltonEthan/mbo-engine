use dbn::{
    FlagSet,
    decode::{DecodeStream, dbn::Decoder},
    record::MboMsg,
};
use fallible_streaming_iterator::FallibleStreamingIterator;
use rtrb::{Consumer, PopError, Producer, PushError, RingBuffer};
use std::{
    hint,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use crate::{
    config::Config,
    enums::Ack,
    stream::{Ext, process_dir},
};

macro_rules! iterstream {
    ($stream:expr, $cfg:expr, $tp:expr) => {{
        let mut dbn_stream = $stream;
        while let Some(mbo) = dbn_stream.next()? {
            if mbo.ts_recv < $cfg.start {
                continue;
            }
            if mbo.ts_recv >= $cfg.end {
                break;
            }
            $tp.dispatch(Mbo::from(mbo));
        }
        anyhow::Ok(())
    }};
}

pub struct RxMsg<MF, AF> {
    pub make_rm: MF,
    pub make_ra: AF,
}

pub struct ThreadPool {
    producers: Vec<Producer<Mbo>>,
    _handles: Vec<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
    mask: usize,
}

impl ThreadPool {
    pub fn new<MF, AF, RM, RA>(rx_msg: RxMsg<MF, AF>, workers: usize, cap: usize) -> Self
    where
        MF: Fn() -> RM + Sync,
        AF: Fn() -> RA + Sync,
        RM: FnMut(Mbo) + Send + 'static,
        RA: FnMut(Ack) + Send + 'static,
    {
        let workers = workers.max(1).next_power_of_two();
        let mask = workers - 1;
        let cap = cap.max(1);
        let running = Arc::new(AtomicBool::new(true));
        let mut producers = Vec::<Producer<Mbo>>::with_capacity(workers);
        let mut handles = Vec::<thread::JoinHandle<()>>::with_capacity(workers);
        for worker_idx in 0..workers {
            let (prod, cons) = RingBuffer::<Mbo>::new(cap);
            producers.push(prod);
            let mut rx_mbo = (rx_msg.make_rm)();
            let mut rx_ack = (rx_msg.make_ra)();
            let running_clone = Arc::clone(&running);
            handles.push(thread::spawn(move || {
                worker_loop(worker_idx, cons, &mut rx_mbo, &mut rx_ack, running_clone)
            }));
        }
        Self {
            producers,
            _handles: handles,
            running,
            mask,
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn dispatch(&mut self, mbo: Mbo) {
        let idx = (mbo.instrument_id as usize) & self.mask;
        let _ = self.producers[idx].push(mbo);
    }

    #[allow(dead_code)]
    #[inline]
    pub fn dispatch_lossless(&mut self, mut mbo: Mbo) {
        let idx = (mbo.instrument_id as usize) & self.mask;
        loop {
            match self.producers[idx].push(mbo) {
                Ok(()) => break,
                Err(PushError::Full(returned)) => {
                    mbo = returned;
                    hint::spin_loop();
                },
            }
        }
    }

    pub fn shutdown(mut self) {
        self.running.store(false, Ordering::Release);
        for h in self._handles.drain(..) {
            let _ = h.join();
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Release);
    }
}

fn worker_loop<RM, RA>(
    _worker_idx: usize,
    mut cons: Consumer<Mbo>,
    rx_mbo: &mut RM,
    rx_ack: &mut RA,
    running: Arc<AtomicBool>,
) where
    RM: FnMut(Mbo),
    RA: FnMut(Ack),
{
    while running.load(Ordering::Acquire) {
        match cons.pop() {
            Ok(mbo) => {
                (rx_mbo)(mbo);
                _ = rx_ack;
            },
            Err(PopError::Empty) => {
                hint::spin_loop();
            },
        }
    }
    while let Ok(mbo) = cons.pop() {
        (rx_mbo)(mbo);
    }
}

#[allow(dead_code)]
pub fn run<MF, AF, RM, RA>(cfg: &Config, rx_msg: RxMsg<MF, AF>) -> anyhow::Result<()>
where
    MF: Fn() -> RM + Sync,
    AF: Fn() -> RA + Sync,
    RM: FnMut(Mbo) + Send + 'static,
    RA: FnMut(Ack) + Send + 'static,
{
    let workers = cfg.workers;
    let qcap = cfg.qcap;
    let mut tp = ThreadPool::new(rx_msg, workers, qcap);
    for (ext, path) in process_dir(cfg.dir.as_path(), cfg.start, cfg.end)? {
        match ext {
            Ext::Zst => iterstream!(Decoder::from_zstd_file(&path)?.decode_stream::<MboMsg>(), cfg, &mut tp)?,
            Ext::Dbn => iterstream!(Decoder::from_file(&path)?.decode_stream::<MboMsg>(), cfg, &mut tp)?,
        }
    }
    tp.shutdown();
    Ok(())
}

#[allow(unused)]
pub struct Mbo {
    ts_recv: u64,
    ts_event: u64,
    publisher_id: u16,
    instrument_id: u32,
    action: i8,
    side: i8,
    price: i64,
    size: u32,
    order_id: u64,
    flags: FlagSet,
}

impl From<&MboMsg> for Mbo {
    fn from(msg: &MboMsg) -> Self {
        Self {
            ts_recv: msg.ts_recv,
            ts_event: msg.hd.ts_event,
            publisher_id: msg.hd.publisher_id,
            instrument_id: msg.hd.instrument_id,
            action: msg.action,
            side: msg.side,
            price: msg.price,
            size: msg.size,
            order_id: msg.order_id,
            flags: msg.flags,
        }
    }
}
