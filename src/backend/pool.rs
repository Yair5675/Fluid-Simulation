use std::{
    ops::{Deref, DerefMut},
    sync::mpsc::{Receiver, TryRecvError, channel, sync_channel},
};

use crate::backend::generic_sender::GenericSender;

/// A RAII guard that automatically sends the value it holds back to the pool that
/// held it.
pub struct Fish<T: Send>(Option<T>, GenericSender<T>);

impl<T: Send> Fish<T> {
    pub fn new(value: T, pool_sender: GenericSender<T>) -> Self {
        Self(Some(value), pool_sender)
    }

    /// Consumes the `Fish` wrapper and returns the value held inside it.
    ///
    /// This effectively releases the value from the pool and allows its memory to
    /// be freed without returning to the pool.
    pub fn into_inner(mut self) -> T {
        self.0.take().expect("Usage of a Fish after it was dropped")
    }
}

impl<T: Send> Deref for Fish<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
            .as_ref()
            .expect("Usage of a Fish after it was dropped")
    }
}

impl<T: Send> DerefMut for Fish<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
            .as_mut()
            .expect("Usage of a Fish after it was dropped")
    }
}

impl<T: Send> Drop for Fish<T> {
    fn drop(&mut self) {
        if let Some(value) = self.0.take() {
            let _ = self.1.send(value); // TODO: Consider logging errors in the future
        }
        // If the value was None the fish was consumed using `into_inner`, which is ok
    }
}

/// A pool of reusable objects. Useful to save memory if the value is very large.
pub struct Pool<T: Send> {
    available_fish: Receiver<T>,
    fish_entry: GenericSender<T>,
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
            fish_entry: GenericSender::Sync(sender),
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
            fish_entry: GenericSender::Async(sender),
        }
    }

    /// Retrieves a value from the pool, blocking until at least one is available.
    ///
    /// # Return Value:
    /// The ownership of a ![fish](https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcSJa8Uri9F5Mv8Em1wSMl8bTO9_ucqruFHbiA&s)
    /// from the pool.
    pub fn get_fish_blocking(&self) -> Fish<T> {
        Fish::new(
            self.available_fish
                .recv()
                .expect("All senders disconnected even though one is saved in the pool itself"),
            self.get_fish_sender(),
        )
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
    pub fn try_get_fish(&self) -> Option<Fish<T>> {
        match self.available_fish.try_recv() {
            Ok(fish) => Some(Fish::new(fish, self.get_fish_sender())),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                panic!("All senders disconnected even though one is saved in the pool itself")
            }
        }
    }

    /// Attempts to retrieve a value from the pool, creating it if one isn't available. This function
    /// never blocks, and guarantees that a fish is returned.
    ///
    /// # Arguments:
    /// * `initializer` - A function that can create a new value for the fish to hold (if necessary).
    ///
    /// # Return Value:
    /// If one is available, the function gives the caller ownership of a new, magnificent, mesmerising
    /// ![fish](https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcSJa8Uri9F5Mv8Em1wSMl8bTO9_ucqruFHbiA&s) from the
    /// pool.
    ///
    /// Otherwise, it makes one for you using the given `initializer`.
    pub fn get_fish_or_init<F>(&self, initializer: F) -> Fish<T>
    where
        F: FnOnce() -> T,
    {
        self.try_get_fish().unwrap_or_else(|| {
            let value = initializer();
            Fish::new(value, self.get_fish_sender())
        })
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
    pub fn get_fish_sender(&self) -> GenericSender<T> {
        self.fish_entry.clone()
    }
}
