use crossbeam_channel::{Sender, Receiver};
use std::collections::HashMap;

pub enum Work {
    Close,
    Closure(Box<dyn FnOnce() + Send + Sync>),
}

pub struct Threadpool {
    sender: Sender<Work>,
    receiver: Receiver<Work>,
    threads: HashMap<std::thread::ThreadId, std::thread::JoinHandle<()>>,
    threads_to_close: (Sender<std::thread::ThreadId>, Receiver<std::thread::ThreadId>),
}

impl Threadpool {
    pub fn new() -> Threadpool {
        Threadpool::new_with_threads(0)
    }

    pub fn new_with_threads(num: usize) -> Threadpool {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let mut threadpool = Threadpool {
            sender: sender,
            receiver: receiver,
            threads: HashMap::new(),
            threads_to_close: crossbeam_channel::unbounded(),
        };

        for _ in 0..num {
            #[cfg(not(target_arch="wasm32"))]
            threadpool.new_thread();
            #[cfg(target_arch="wasm32")]
            let a:i32;
        }

        return threadpool;
    }

    pub fn num_threads(&self) -> usize {
        self.threads.len()
    }

    pub fn new_thread(&mut self) {
        let work_receiver = self.receiver.clone();
        let send_close = self.threads_to_close.0.clone();
        let thread = std::thread::spawn(move || {
            loop {
                let maybe_work = work_receiver.try_recv();
                match maybe_work {
                    Ok(work) => {   
                        match work {
                            Work::Close => { send_close.send(std::thread::current().id()).unwrap(); break }
                            Work::Closure(f) => { f(); }
                        }
                    },
                    Err(_) => {}
                }
            }
        });
        let id_thread = thread.thread().id();
        self.threads.insert(
            id_thread,
            thread
        );
    }

    pub fn delete_thread(&mut self) {
        if self.num_threads() == 0 { return }
        self.sender.send(Work::Close).unwrap();
        let id = self.threads_to_close.1.recv().unwrap();
        let thread = self.threads.remove(&id).unwrap();
        thread.join().unwrap();
    }

    pub fn send_work<F: 'static + FnOnce() + Send + Sync>(&mut self, f: F) {
        self.sender.send(Work::Closure(Box::new(f))).unwrap();
    }
    
    pub unsafe fn send_work_unsafe_boxed<'a>(&mut self, f: Box<dyn FnOnce() + 'a + Send + Sync>) {
        let f = std::mem::transmute(f);
        self.sender.send(Work::Closure(f)).unwrap();
    }
    
    pub unsafe fn send_work_unsafe<F: FnOnce() + Send + Sync>(&mut self, f: F) {
        self.send_work_unsafe_boxed(Box::new(move || f()));
    }

    pub fn wait(&mut self) {
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(self.num_threads() + 1));
        for _ in 0..self.num_threads() {
            let barrier = barrier.clone();
            unsafe {
                self.send_work_unsafe(move || {
                    barrier.wait();
                });
            }
        }
        barrier.wait();
    }
}

impl Drop for Threadpool {
    fn drop(&mut self) {
        while self.num_threads() > 0 {
            self.delete_thread();
        }
    }
}