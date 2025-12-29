use dbn::{
    FlagSet,
    decode::{DecodeStream, dbn::Decoder},
    record::MboMsg,
};
use fallible_streaming_iterator::FallibleStreamingIterator;
use hashbrown::HashMap;
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
    orderbook::book::{Book, LobMbo},
    stream::{Ext, process_dir},
};

/// Helper macro to iterate a DBN `DecodeStream` within the configured time window
/// and dispatch each message to a [`ThreadPool`].
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
            $tp.dispatch_lossless(Mbo::from(mbo));
        }
        anyhow::Ok(())
    }};
}

/// Factory struct for creating per worker receivers/handlers.
///
/// Each worker thread gets its own:
/// - `RM`: handler for [`Mbo`] messages
/// - `RA`: handler for [`Ack`] messages
pub struct RxMsg<MF, AF> {
    pub make_rm: MF,
    pub make_ra: AF,
}

/// A simple fixed-size thread pool that routes [`Mbo`] messages to workers,
/// SPSC queue.
pub struct ThreadPool {
    producers: Vec<Producer<Mbo>>,
    _handles: Vec<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
    mask: usize,
}

impl ThreadPool {
    /// Create a new thread pool.
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
            let mut books: HashMap<u32, Book> = HashMap::new();
            let running_clone = Arc::clone(&running);
            handles.push(thread::spawn(move || {
                worker_loop(worker_idx, cons, &mut rx_mbo, &mut rx_ack, &mut books, running_clone)
            }));
        }
        Self {
            producers,
            _handles: handles,
            running,
            mask,
        }
    }

    /// Dispatch a message, spin waiting until it can be enqueued.
    ///
    /// This ensures delivery to the worker queue
    #[inline]
    pub fn dispatch_lossless(&mut self, mut mbo: Mbo) {
        let idx = (mbo.instrument_id as usize) & self.mask;
        while self.running.load(Ordering::Acquire) {
            match self.producers[idx].push(mbo) {
                Ok(()) => return,
                Err(PushError::Full(returned)) => {
                    mbo = returned;
                    hint::spin_loop();
                },
            }
        }
        panic!("ThreadPool stopped while dispatching");
    }

    /// Stop all workers and join their threads.
    pub fn shutdown(mut self) {
        self.running.store(false, Ordering::Release);
        for h in self._handles.drain(..) {
            h.join().unwrap();
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Release);
    }
}

/// Worker loop for a single consumer queue.
///
/// # Performance note
/// Uses spin waiting when the queue is empty.
fn worker_loop<RM, RA>(
    _worker_idx: usize,
    mut cons: Consumer<Mbo>,
    rx_mbo: &mut RM,
    rx_ack: &mut RA,
    books: &mut HashMap<u32, Book>,
    running: Arc<AtomicBool>,
) where
    RM: FnMut(Mbo),
    RA: FnMut(Ack),
{
    while running.load(Ordering::Acquire) {
        match cons.pop() {
            Ok(mbo) => {
                let book = books.entry(mbo.instrument_id).or_insert_with(Book::new);
                book.apply(LobMbo::from(&mbo));
                (rx_mbo)(mbo);
                _ = rx_ack;
            },
            Err(PopError::Empty) => {
                hint::spin_loop();
            },
        }
    }
    while let Ok(mbo) = cons.pop() {
        let book = books.entry(mbo.instrument_id).or_insert_with(Book::new);
        book.apply(LobMbo::from(&mbo));
        (rx_mbo)(mbo);
    }
}

/// Run the DBN stream processing pipeline:
/// - builds a [`ThreadPool`]
/// - iterates DBN/ZST files in `cfg.dir`
/// - dispatches each record to a worker
/// - shuts down the pool at the end
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

/// Owned, compact MBO record used by the worker threads.
#[allow(unused)]
pub struct Mbo {
    pub ts_recv: u64,
    pub ts_event: u64,
    pub publisher_id: u16,
    pub instrument_id: u32,
    pub action: i8,
    pub side: i8,
    pub price: i64,
    pub size: u32,
    pub order_id: u64,
    pub flags: FlagSet,
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
