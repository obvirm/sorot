use rayon::prelude::*;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TilePriority {
    Background = 0,
    Foreground = 1,
    Visible = 2,
}

pub struct WorkerPool {
    pool: rayon::ThreadPool,
}

impl WorkerPool {
    pub fn new(num_threads: usize) -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .expect("failed to build thread pool");
        Self { pool }
    }

    pub fn num_threads(&self) -> usize {
        self.pool.current_num_threads()
    }

    /// Execute a batch of tile jobs in parallel.
    /// `tiles` is a slice of work items, `priority` determines scheduling order.
    pub fn schedule<T, F>(&self, tiles: &[T], priority: TilePriority, f: F)
    where
        T: Sync,
        F: Fn(&T, TilePriority) + Send + Sync,
    {
        let prio = Arc::new(priority);
        self.pool.install(|| {
            tiles.par_iter().for_each(|tile| {
                f(tile, *prio);
            });
        });
    }

    /// Execute two-phase scheduling: high-priority tiles first, then low-priority.
    pub fn schedule_priority<T, F>(
        &self,
        visible: &[T],
        foreground: &[T],
        background: &[T],
        f: F,
    ) where
        T: Sync,
        F: Fn(&T, TilePriority) + Send + Sync,
    {
        if !visible.is_empty() {
            self.schedule(visible, TilePriority::Visible, &f);
        }
        if !foreground.is_empty() {
            self.schedule(foreground, TilePriority::Foreground, &f);
        }
        if !background.is_empty() {
            self.schedule(background, TilePriority::Background, &f);
        }
    }
}

impl Default for WorkerPool {
    fn default() -> Self {
        let threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        Self::new(threads)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_schedule_parallel() {
        let pool = WorkerPool::new(4);
        let counter = Arc::new(AtomicUsize::new(0));
        let tiles: Vec<u32> = (0..100).collect();

        let c = counter.clone();
        pool.schedule(&tiles, TilePriority::Visible, move |_, _| {
            c.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(counter.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn test_schedule_priority() {
        let pool = WorkerPool::new(4);
        let order = Arc::new(std::sync::Mutex::new(Vec::new()));

        let v: Vec<u32> = vec![1];
        let f: Vec<u32> = vec![2];
        let b: Vec<u32> = vec![3];

        let o = order.clone();
        pool.schedule_priority(&v, &f, &b, move |val, prio| {
            o.lock().unwrap().push((*val, prio as u8));
        });

        let result = order.lock().unwrap();
        assert!(!result.is_empty());
    }
}
