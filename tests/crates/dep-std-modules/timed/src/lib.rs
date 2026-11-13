use std::cell::OnceCell;
use std::collections::{BTreeMap, VecDeque};
use std::marker::PhantomData;
use std::net::Ipv4Addr;
use std::ops::ControlFlow;
use std::sync::{LazyLock, Once};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

static READY: Once = Once::new();

static TABLE: LazyLock<BTreeMap<u32, &'static str>> = LazyLock::new(|| BTreeMap::from([(2, "two"), (1, "one")]));

pub struct Job<T> {
    pub deadline: Instant,
    pub queue: VecDeque<u32>,
    pub marker: PhantomData<T>,
    pub once: OnceCell<u32>,
}

pub fn job() -> Job<u8> {
    let mut queue = VecDeque::new();
    queue.push_back(1);
    queue.push_front(0);
    Job { deadline: Instant::now() + Duration::from_millis(1500), queue, marker: PhantomData, once: OnceCell::new() }
}

impl<T> Job<T> {
    pub fn report(&self) -> String {
        READY.call_once(|| println!("ready"));
        READY.call_once(|| println!("again"));
        let left = self.deadline.saturating_duration_since(Instant::now());
        let names: Vec<&str> = TABLE.values().copied().collect();
        let after_epoch = SystemTime::now().duration_since(UNIX_EPOCH).is_ok();
        let first = *self.once.get_or_init(|| 41) + 1;
        format!("{:?} {} {:?} {} {} {}", self.queue, left <= Duration::from_secs(2), names, after_epoch, first, Ipv4Addr::LOCALHOST)
    }
}

pub fn wait() -> Duration {
    Duration::from_secs(3) / 2
}

/// The first queued value at or over `limit`, through `?` on `ControlFlow`.
pub fn first_over(queue: &VecDeque<u32>, limit: u32) -> ControlFlow<u32> {
    for &x in queue {
        if x >= limit { ControlFlow::Break(x)?; }
    }
    ControlFlow::Continue(())
}
