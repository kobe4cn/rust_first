use anyhow::Result;
use std::{
    collections::VecDeque,
    mem::swap,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
pub struct Share<T> {
    queue: Mutex<VecDeque<T>>,
    available: Condvar,
    senders: AtomicUsize,
    receivers: AtomicUsize,
}

pub struct Sender<T> {
    shared: Arc<Share<T>>,
}

pub struct Receiver<T> {
    shared: Arc<Share<T>>,
    cached: VecDeque<T>,
}

impl<T> Sender<T> {
    pub fn send(&self, item: T) -> Result<()> {
        if self.total_receivers() == 0 {
            return Err(anyhow::anyhow!("no receiver"));
        }

        let was_empty = {
            let mut inner = self.shared.queue.lock().unwrap();
            let empty = inner.is_empty();
            inner.push_back(item);
            empty
        };
        //只有对列从空到非空才通知

        if was_empty {
            self.shared.available.notify_one();
        }

        Ok(())
    }
    pub fn total_receivers(&self) -> usize {
        self.shared.receivers.load(Ordering::SeqCst)
    }
    pub fn total_queue_size(&self) -> usize {
        self.shared.queue.lock().unwrap().len()
    }
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Result<T> {
        if let Some(v) = self.cached.pop_front() {
            return Ok(v);
        }

        let mut inner = self.shared.queue.lock().unwrap();
        loop {
            match inner.pop_front() {
                Some(item) => {
                    if !inner.is_empty() {
                        swap(&mut self.cached, &mut inner);
                    }
                    return Ok(item);
                }
                None if self.total_senders() == 0 => return Err(anyhow::anyhow!("no sender")),
                None => {
                    inner = self
                        .shared
                        .available
                        .wait(inner)
                        .map_err(|_| anyhow::anyhow!("lock poisoned"))?;
                }
            }
        }
    }
    pub fn total_senders(&self) -> usize {
        self.shared.senders.load(Ordering::SeqCst)
    }
}

impl<T> Iterator for Receiver<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.recv().ok()
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.shared.senders.fetch_add(1, Ordering::AcqRel);
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}
impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let old = self.shared.senders.fetch_sub(1, Ordering::AcqRel);
        if old <= 1 {
            self.shared.available.notify_all();
        }
    }
}
impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.shared.receivers.fetch_sub(1, Ordering::AcqRel);
    }
}
pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
    let shared = Share::default();
    let shared = Arc::new(shared);
    let sender = Sender {
        shared: shared.clone(),
    };
    let receiver = Receiver {
        shared,
        cached: VecDeque::with_capacity(INIT_SIZE),
    };
    (sender, receiver)
}
const INIT_SIZE: usize = 32;
impl<T> Default for Share<T> {
    fn default() -> Self {
        Self {
            queue: Mutex::new(VecDeque::with_capacity(INIT_SIZE)),
            available: Condvar::new(),
            senders: AtomicUsize::new(1),
            receivers: AtomicUsize::new(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;

    #[test]
    fn test_mpsc_channel() {
        let (tx, mut rx) = unbounded();
        tx.send(1).unwrap();
        assert_eq!(rx.recv().unwrap(), 1);
    }

    #[test]
    fn multiple_senders_should_work() {
        let (s, mut r) = unbounded();
        let s1 = s.clone();
        let s2 = s.clone();
        let t = thread::spawn(move || {
            s.send(1).unwrap();
        });
        let t2 = thread::spawn(move || {
            s1.send(2).unwrap();
        });
        let t3 = thread::spawn(move || {
            s2.send(3).unwrap();
        });
        for handle in [t, t2, t3] {
            handle.join().unwrap();
        }
        let mut result = [r.recv().unwrap(), r.recv().unwrap(), r.recv().unwrap()];
        result.sort();
        assert_eq!(result, [1, 2, 3]);
    }

    #[test]
    fn receiver_should_be_block_when_nothing_to_read() {
        let (s, r) = unbounded();

        let s1 = s.clone();
        thread::spawn(move || {
            for (idx, item) in r.into_iter().enumerate() {
                assert_eq!(idx, item);
            }
            assert!(false);
        });

        thread::spawn(move || {
            for i in 0..100usize {
                s.send(i).unwrap();
            }
        });

        thread::sleep(Duration::from_secs(1));

        for i in 100..200usize {
            s1.send(i).unwrap();
        }

        thread::sleep(Duration::from_secs(2));
        assert_eq!(s1.total_queue_size(), 0);
    }

    #[test]
    fn last_sender_drop_should_error_when_receive() {
        let (s, mut r) = unbounded();
        let s1 = s.clone();
        let senders = [s, s1];
        let total = senders.len();
        for sender in senders {
            thread::spawn(move || {
                sender.send("hello").unwrap();
            })
            .join()
            .unwrap();
        }
        //已经生产的两个生成者数据可以被读取
        for _ in 0..total {
            r.recv().unwrap();
        }

        //最后一个生成者被drop，读取会失败
        assert!(r.recv().is_err());
    }

    #[test]
    fn receiver_drop_should_error_when_send() {
        let (s, _) = unbounded();
        let (s1, s2) = {
            let s1 = s.clone();
            let s2 = s.clone();
            (s1, s2)
        };
        assert!(s1.send("hello").is_err());
        assert!(s2.send("hello").is_err());
    }

    #[test]
    fn channel_cache_should_wrok() {
        let (s, mut r) = unbounded();
        for i in 0..100usize {
            s.send(i).unwrap();
        }
        assert!(r.cached.is_empty());

        assert_eq!(r.recv().unwrap(), 0);
        assert_eq!(r.cached.len(), 99);
        assert_eq!(s.total_queue_size(), 0);

        for (idx, i) in r.into_iter().take(9).enumerate() {
            assert_eq!(idx + 1, i);
        }
    }
}
