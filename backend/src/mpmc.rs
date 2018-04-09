use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard, PoisonError};

use futures::{Async, Stream};
use futures::task::{self, Task};

#[derive(Debug)]
pub struct Poisoned;

impl<'a, T> From<PoisonError<MutexGuard<'a, T>>> for Poisoned {
    fn from(_: PoisonError<MutexGuard<'a, T>>) -> Self {
        Poisoned
    }
}

#[derive(Debug)]
struct Inner<T: Clone> {
    /// The number of streams reading from this queue
    stream_count: usize,
    /// The list of tasks waiting for someone to write to the end of the queue
    waiting: Vec<Option<*mut Task>>,
    /// A global ID of the last item added to the queue. Modelled as a simple counter that wraps
    last_added: u64,
    /// The items in the queue. Each one is stored with the number of streams that have yielded
    /// that value.
    queue: VecDeque<(usize, T)>,
}

/// A multiple producer, multiple consumer queue that is futures aware.
#[derive(Debug)]
pub struct Mpmc<T: Clone> {
    inner: Mutex<Inner<T>>,
}

impl<T: Clone> Mpmc<T> {
    /// Create a new queue.
    pub fn new() -> Self {
        Mpmc {
            inner: Mutex::new(Inner {
                stream_count: 0,
                waiting: Vec::new(),
                last_added: 0,
                queue: VecDeque::new(),
            })
        }
    }

    /// Create a new streamer that reads from the queue.
    pub fn stream<'a>(&'a self) -> Result<MpmcStream<'a, T>, Poisoned> {
        let (last_added, wait_index) = {
            let mut inner = self.inner.lock()?;
            inner.stream_count += 1;
            inner.waiting.push(None);
            (inner.last_added, inner.waiting.len() - 1)
        };
        Ok(MpmcStream { inner: self, last_added, wait_index })
    }

    /// Writes an item to the queue, waking up any stream that was listening
    pub fn send(&self, item: T) -> Result<(), Poisoned> {
        let mut inner = self.inner.lock()?;

        // first, push task onto the queue
        inner.last_added += 1;
        inner.queue.push_back((0, item));

        // second, tell everyone else about it
        for task in inner.waiting.iter_mut().filter_map(|x| x.take()) {
            unsafe { Box::from_raw(task).notify(); }
        }

        Ok(())
    }
}

/// The consumer end of the multiple producer, multiple consumer queue that is futures aware.
pub struct MpmcStream<'a, T: 'a + Clone> {
    inner: &'a Mpmc<T>,
    last_added: u64,
    wait_index: usize,
}

impl<'a, T: 'a + Clone> Stream for MpmcStream<'a, T> {
    type Item = T;
    type Error = Poisoned;

    fn poll(&mut self) -> Result<Async<Option<T>>, Self::Error> {
        let mut inner = self.inner.inner.lock()?;

        // first, were we previously blocked on this lock?
        if let Some(ptr) = inner.waiting[self.wait_index].take() {
            unsafe { drop(Box::from_raw(ptr)); }
        }

        // second, find where the queue is up to now
        let offset = inner.last_added - self.last_added;

        // third, block if there are no new elements
        if offset == 0 {
            let me = Box::new(task::current());
            let me = Box::into_raw(me);

            inner.waiting[self.wait_index] = Some(me).take();
            return Ok(Async::NotReady);
        }

        // finally, as there are new elements, update internal state and then yield one of them
        let len = inner.queue.len();
        let stream_count = inner.stream_count;

        let (output, pop) = {
            let (ref mut counter, ref item) = inner.queue[len - offset as usize];
            let output = item.clone();

            *counter += 1;
            let pop = *counter >= stream_count && offset as usize == len;

            (output, pop)
        };

        self.last_added += 1;

        if pop {
            inner.queue.pop_front();
        }

        Ok(Async::Ready(Some(output)))
    }
}
