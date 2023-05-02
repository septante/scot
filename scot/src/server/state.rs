//! Every server needs a [`State`] type, which must, at minimum, be able to
//! generate IDs for clients that join. The [`State`] must also be [`Send`],
//! but otherwise nothing is assumed about it. In order to provide mutability
//! across threads, one option is to define an inner type that implements
//! [`State`], and wrap it in an [`Arc<Mutex<T>>`]. In this case, the impl
//! for the outer type will be automatically generated. Another option, for
//! applications that want more fine-grained access to data, is to have
//! multiple fields, each of type [`Arc<Mutex<T>>], or even a field
//! whose type is [`Vec<Arc<Mutex<T>>>].

use std::sync::Arc;

/// Trait for server state type.
///
/// Type parameter is the type used for client IDs.
pub trait State {
    /// The type used to uniquely identify clients.
    type ClientID;

    /// Function to be called when a new client connects. Must return a new,
    /// unique ID.
    fn on_join(&mut self) -> Self::ClientID;
}

impl<T> State for Arc<std::sync::Mutex<T>>
where
    T: State,
{
    type ClientID = T::ClientID;

    fn on_join(&mut self) -> Self::ClientID {
        self.lock().unwrap().on_join()
    }
}

impl<T> State for Arc<parking_lot::Mutex<T>>
where
    T: State,
{
    type ClientID = T::ClientID;

    fn on_join(&mut self) -> Self::ClientID {
        self.lock().on_join()
    }
}
