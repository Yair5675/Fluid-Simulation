//! A module that contains a type which abstracts between a [`SyncSender`] and a [`Sender`]

use std::sync::mpsc::{SendError, Sender, SyncSender, TrySendError};

/// A sender of values to some receiver. Abstracts over asynchronous and synchronous senders.
#[derive(Debug)]
pub enum GenericSender<T: Send> {
    Async(Sender<T>),
    Sync(SyncSender<T>),
}

impl<T: Send> Clone for GenericSender<T> {
    fn clone(&self) -> Self {
        match self {
            GenericSender::Async(sender) => GenericSender::Async(sender.clone()),
            GenericSender::Sync(sync_sender) => GenericSender::Sync(sync_sender.clone()),
        }
    }
}

impl<T: Send> GenericSender<T> {
    /// Sends the given value to the designated receiver.
    ///
    /// The behavior of the function depends on the underlying implementation:
    /// * For the `Async` variant, the function will return immediately, even if the
    ///   value could not be sent because the receiver disconnected.
    /// * For the `Sync` variant, the function will block until either enough space in
    ///   the receiver will be available, or the receiver disconnected (in which case
    ///   an error will be returned).
    ///
    /// If you wish to consistently send data asynchronously, see [`GenericSender::send_async`].
    ///
    /// # Arguments:
    /// * `value` - The value sent through the channel to some receiver.
    ///
    /// # Return Value:
    /// A unit type of the value was sent successfully, otherwise a [`SendError`] containing
    /// `value`.
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        match self {
            GenericSender::Async(sender) => sender.send(value),
            GenericSender::Sync(sync_sender) => sync_sender.send(value),
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
            GenericSender::Async(sender) => sender
                .send(value)
                .map_err(|async_send_err| TrySendError::Disconnected(async_send_err.0)),
            GenericSender::Sync(sync_sender) => sync_sender.try_send(value),
        }
    }
}
