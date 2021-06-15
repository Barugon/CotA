use std::{
  sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Mutex, Weak,
  },
  thread,
};

pub type Cancel = Box<dyn Fn() -> bool>;

/// A `ThreadPool` contains a number of worker threads to be used for concurrently executing closures.
pub struct ThreadPool {
  sender: mpsc::Sender<Message>,
  workers: Vec<Worker>,
  cancellables: Vec<Cancellable>,
}

impl ThreadPool {
  /// Create a new `ThreadPool` with a number of threads specified by `size`.
  ///
  /// ### Panics
  /// If size is zero.
  pub fn new(size: usize) -> ThreadPool {
    assert!(size > 0);

    let (sender, receiver) = mpsc::channel();
    let receiver = Arc::new(Mutex::new(receiver));
    let mut workers = Vec::with_capacity(size);

    for id in 0..size {
      workers.push(Worker::new(id, Arc::clone(&receiver)));
    }

    ThreadPool {
      sender,
      workers,
      cancellables: Vec::new(),
    }
  }

  /// Execute a closure on one of the pool's threads.
  ///
  /// Places the closure in a queue until a thread is available to process it.
  ///
  /// ### Returns
  /// A `Task` object.
  pub fn exec<F, R>(&mut self, job: F) -> Task<R>
  where
    F: FnOnce(Cancel) -> Option<R> + Send + 'static,
    R: Send + 'static,
  {
    // Create a task.
    let cancel = Arc::new(AtomicBool::new(false));
    let result = Arc::new(Mutex::new(None));
    let task = Task::new(Arc::downgrade(&cancel), Arc::clone(&result));

    // Box up the job.
    let job = Box::new(move || {
      if let Ok(mut result) = result.lock() {
        *result = job(Box::new(move || cancel.load(Ordering::Relaxed)));
      }
    });

    // Put the job on the queue.
    self.sender.send(Message::Execute(job)).unwrap();

    // Cleanup the cancellables and place the new one on the list.
    self.cancellables.retain(|cancellable| cancellable.valid());
    self.cancellables.push(task.cancellable.clone());

    task
  }
}

impl Drop for ThreadPool {
  fn drop(&mut self) {
    // Send terminate messages, one for each thread.
    for _ in &self.workers {
      self.sender.send(Message::Terminate).unwrap();
    }

    // Cancel all jobs.
    for cancellable in &mut self.cancellables {
      cancellable.cancel();
    }

    // Wait for all threads to join.
    for worker in &mut self.workers {
      if let Some(thread) = worker.thread.take() {
        thread.join().unwrap();
      }
    }
  }
}

/// Represents a `ThreadPool` job with an optional result and offers the opportunity to cancel the work.
#[derive(Clone)]
pub struct Task<T> {
  cancellable: Cancellable,
  result: Arc<Mutex<Option<T>>>,
}

impl<T> Task<T> {
  fn new(cancel: Weak<AtomicBool>, result: Arc<Mutex<Option<T>>>) -> Task<T> {
    Task {
      cancellable: Cancellable { cancel },
      result,
    }
  }

  /// Cancel this task.
  pub fn cancel(&mut self) {
    self.cancellable.cancel();
  }

  /// Wait for this task to conclude.
  pub fn wait(&self) {
    while self.cancellable.valid() {
      thread::yield_now();
    }
  }

  /// Wait for this task to conclude and get the result.
  pub fn get(&mut self) -> Option<T> {
    self.wait();
    self.result.lock().unwrap().take()
  }
}

#[derive(Clone)]
struct Cancellable {
  cancel: Weak<AtomicBool>,
}

impl Cancellable {
  pub fn cancel(&mut self) {
    if let Some(cancel) = self.cancel.upgrade() {
      cancel.store(true, Ordering::Relaxed);
    }
  }

  pub fn valid(&self) -> bool {
    self.cancel.upgrade().is_some()
  }
}

type Job = Box<dyn FnOnce() + Send>;

enum Message {
  Execute(Job),
  Terminate,
}

struct Worker {
  thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
  fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
    Worker {
      thread: Some(
        thread::Builder::new()
          .name(format!("Worker {}", id))
          .spawn(move || loop {
            let msg = receiver.lock().unwrap().recv().unwrap();
            match msg {
              Message::Execute(job) => {
                job();
              }
              Message::Terminate => {
                break;
              }
            }
          })
          .unwrap(),
      ),
    }
  }
}
