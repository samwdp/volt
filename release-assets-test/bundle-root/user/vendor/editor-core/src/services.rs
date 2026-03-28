use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

type BoxedService = Box<dyn Any + Send + Sync>;

/// Type-indexed registry for runtime services shared across editor subsystems.
#[derive(Default)]
pub struct ServiceRegistry {
    services: HashMap<TypeId, BoxedService>,
}

impl ServiceRegistry {
    /// Creates an empty service registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts or replaces a service instance.
    pub fn insert<T>(&mut self, service: T) -> Option<T>
    where
        T: Send + Sync + 'static,
    {
        self.services
            .insert(TypeId::of::<T>(), Box::new(service))
            .and_then(|service| service.downcast::<T>().ok().map(|service| *service))
    }

    /// Returns a shared reference to a registered service.
    pub fn get<T>(&self) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.services
            .get(&TypeId::of::<T>())
            .and_then(|service| service.downcast_ref::<T>())
    }

    /// Returns a mutable reference to a registered service.
    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Send + Sync + 'static,
    {
        self.services
            .get_mut(&TypeId::of::<T>())
            .and_then(|service| service.downcast_mut::<T>())
    }

    /// Removes and returns a registered service.
    pub fn remove<T>(&mut self) -> Option<T>
    where
        T: Send + Sync + 'static,
    {
        self.services
            .remove(&TypeId::of::<T>())
            .and_then(|service| service.downcast::<T>().ok().map(|service| *service))
    }

    /// Returns whether a service of the requested type is registered.
    pub fn contains<T>(&self) -> bool
    where
        T: Send + Sync + 'static,
    {
        self.services.contains_key(&TypeId::of::<T>())
    }

    /// Returns the number of registered services.
    pub fn len(&self) -> usize {
        self.services.len()
    }

    /// Returns whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.services.is_empty()
    }
}
