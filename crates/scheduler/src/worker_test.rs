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
