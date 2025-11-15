use std::sync::mpsc::{
    channel, sync_channel, Receiver, SendError, Sender, SyncSender, TryRecvError, TrySendError,
};

/// The transmittor to the pool. Abstracts over asynchronous and synchronous senders.
#[derive(Debug)]
pub enum PoolSender<T: Send> {
    Async(Sender<T>),
    Sync(SyncSender<T>),
}

impl<T: Send> Clone for PoolSender<T> {
    fn clone(&self) -> Self {
        match self {
            PoolSender::Async(sender) => PoolSender::Async(sender.clone()),
            PoolSender::Sync(sync_sender) => PoolSender::Sync(sync_sender.clone()),
        }
    }
}

impl<T: Send> PoolSender<T> {
    /// Sends the given value to the designated receiver.
    ///
    /// The behavior of the function depends on the underlying implementation:
    /// * For the `Async` variant, the function will return immediately, even if the
    ///   value could not be sent because the receiver disconnected.
    /// * For the `Sync` variant, the function will block until either enough space in
    ///   the receiver will be available, or the receiver disconnected (in which case
    ///   an error will be returned).
    ///
    /// If you wish to consistently send data asynchronously, see [`PoolSender::send_async`].
    ///
    /// # Arguments:
    /// * `value` - The value sent through the channel to some receiver.
    ///
    /// # Return Value:
    /// A unit type of the value was sent successfully, otherwise a [`SendError`] containing
    /// `value`.
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        match self {
            PoolSender::Async(sender) => sender.send(value),
            PoolSender::Sync(sync_sender) => sync_sender.send(value),
        }
    }

    /// Sends the given value without blocking (asynchronously).
    ///
    /// This is the normal behavior for the `Async` variant, but for the `Sync` variant
    /// [`SyncSender::try_send`] is called.
    /// To abstract over the two senders, the error type is [`TrySendError`], as the `Async`
    /// variant's only possible error is caused due to receiver disconnection (which is a variant)
    /// of `TrySendError`.
    /// # Arguments:
    /// * `value` - The value sent through the channel to some receiver.
    ///
    /// # Return Value:
    /// A unit type of the value was sent successfully, otherwise a `TrySendError` containing
    /// `value`.
    pub fn send_async(&self, value: T) -> Result<(), TrySendError<T>> {
        match self {
            PoolSender::Async(sender) => sender
                .send(value)
                .map_err(|async_send_err| TrySendError::Disconnected(async_send_err.0)),
            PoolSender::Sync(sync_sender) => sync_sender.try_send(value),
        }
    }
}

/// A pool of reusable objects. Useful to save memory if the value is very large.
pub struct Pool<T: Send> {
    available_fish: Receiver<T>,
    fish_entry: PoolSender<T>,
}

impl<T: Send> Pool<T> {
    /// Creates a new pool that can store up to `max_fish` values.
    ///
    /// If users of the pool will attempt to put more values than `max_fish`,
    /// the function will block until another user takes a fish.
    ///
    /// # Arguments:
    /// * `max_fish` - Maximum number of values which the pool can hold at a single instance.
    ///
    /// # Return Value:
    /// A new `Pool` instance with the given capacity limitation.
    pub fn new_bounded(max_fish: usize) -> Self {
        let (sender, receiver) = sync_channel(max_fish);
        Self {
            available_fish: receiver,
            fish_entry: PoolSender::Sync(sender),
        }
    }

    /// Creates a new pool that can store an unlimited number of values.
    ///
    /// # Return Value:
    /// A new unlimited `Pool` instance.
    pub fn new_unbounded() -> Self {
        let (sender, receiver) = channel();
        Self {
            available_fish: receiver,
            fish_entry: PoolSender::Async(sender),
        }
    }

    /// Retrieves a value from the pool, blocking until at least one is available.
    ///
    /// # Return Value:
    /// The ownership of a ![fish](https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcSJa8Uri9F5Mv8Em1wSMl8bTO9_ucqruFHbiA&s)
    /// from the pool.
    pub fn get_fish_blocking(&self) -> T {
        self.available_fish
            .recv()
            .expect("All senders disconnected even though one is saved in the pool itself")
    }

    /// Attempts to retrieve a value from the pool. This function never blocks, and if
    /// no fish is available it just returns `None`.
    /// 
    /// # Return Value:
    /// If one is available, the function gives the caller ownership of a new, magnificent, mesmerising
    /// ![fish](https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcSJa8Uri9F5Mv8Em1wSMl8bTO9_ucqruFHbiA&s) from the
    /// pool (wrapped in `Some`).
    /// 
    /// If one isn't available, `None` is returned.
    pub fn try_get_fish(&self) -> Option<T> {
        match self.available_fish.try_recv() {
            Ok(fish) => Some(fish),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                panic!("All senders disconnected even though one is saved in the pool itself")
            }
        }
    }

    /// Creates a new [`PoolSender`] through which new fish can be sent to the pool, either to populate
    /// it or return a fish that was previously in it.
    ///
    /// Note that if the pool was created with the [`Pool::new_bounded`] function, this function
    /// will block until enough space in the pool is available.
    ///
    /// ![](https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcRQgewMwfqyqrWsb-77rHHFEt6ApfYul31ERw&s)
    /// # Return Value:
    /// A `PoolSender` through which values can be passed to the pool.
    pub fn get_fish_sender(&self) -> PoolSender<T> {
        self.fish_entry.clone()
    }
}
