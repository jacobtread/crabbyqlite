use std::mem::swap;

use gpui::{App, AppContext, Entity, SharedString, Task};

/// Asynchronously loaded resource
pub enum AsyncResource<T> {
    /// Resource load has not been triggered yet
    Idle,
    /// The resource is currently loading in a background task
    Loading(#[allow(unused)] Task<()>),
    /// The loaded value
    Loaded(T),
    /// Error outcome from loading the async resource
    Error(SharedString),
}

impl<T: 'static> AsyncResource<T> {
    pub fn new(cx: &mut App) -> Entity<AsyncResource<T>> {
        cx.new(|_| Self::Idle)
    }

    #[allow(unused)]
    pub fn take_value<C: AppContext>(this: &Entity<Self>, cx: &mut C) -> C::Result<Option<T>> {
        this.update(cx, |this, _cx| {
            let mut value = AsyncResource::Idle;
            swap(&mut value, this);
            match value {
                AsyncResource::Loaded(value) => Some(value),
                _ => None,
            }
        })
    }

    #[allow(unused)]
    pub fn set_value<C: AppContext>(this: &Entity<Self>, cx: &mut C, value: T) {
        this.update(cx, |this, cx| {
            *this = AsyncResource::Loaded(value);
            cx.notify();
        });
    }

    pub fn set_idle<C: AppContext>(this: &Entity<Self>, cx: &mut C) {
        this.update(cx, |this, cx| {
            *this = AsyncResource::Idle;
            cx.notify();
        });
    }

    pub fn load<C, F, Fut>(this: &Entity<Self>, cx: &mut C, loader: F)
    where
        C: AppContext,
        F: FnOnce() -> Fut + 'static,
        Fut: Future<Output = Result<T, anyhow::Error>> + 'static,
    {
        this.update(cx, |this, cx| {
            // Revert to IDLE state (Drops the async task triggering its abort logic)
            *this = AsyncResource::Idle;
            cx.notify();

            // Spawn the loader background task
            let task = cx.spawn(async move |this, cx| {
                let result = loader().await;

                _ = this.update(cx, |this, cx| {
                    *this = match result {
                        Ok(value) => AsyncResource::Loaded(value),
                        Err(error) => AsyncResource::Error(format!("{error:?}").to_string().into()),
                    };
                    cx.notify();
                });
            });

            *this = AsyncResource::Loading(task);
            cx.notify();
        });
    }

    pub fn maybe_load<C, F, Fut>(this: &Entity<Self>, cx: &mut C, loader: F)
    where
        C: AppContext,
        F: FnOnce() -> Fut + 'static,
        Fut: Future<Output = Result<Option<T>, anyhow::Error>> + 'static,
    {
        this.update(cx, |this, cx| {
            // Revert to IDLE state (Drops the async task triggering its abort logic)
            *this = AsyncResource::Idle;
            cx.notify();

            // Spawn the loader background task
            let task = cx.spawn(async move |this, cx| {
                let result = loader().await;

                _ = this.update(cx, |this, cx| {
                    *this = match result {
                        Ok(Some(value)) => AsyncResource::Loaded(value),
                        Ok(None) => AsyncResource::Idle,
                        Err(error) => AsyncResource::Error(error.to_string().into()),
                    };
                    cx.notify();
                });
            });

            *this = AsyncResource::Loading(task);
            cx.notify();
        });
    }
}

/// Extension for Entity<AsyncResource<T>> to make calling .load easier
pub trait AsyncResourceEntityExt<T>
where
    Self: Sized,
{
    fn load<C, F, Fut>(&self, cx: &mut C, loader: F)
    where
        C: AppContext,
        F: FnOnce() -> Fut + 'static,
        Fut: Future<Output = Result<T, anyhow::Error>> + 'static;

    fn maybe_load<C, F, Fut>(&self, cx: &mut C, loader: F)
    where
        C: AppContext,
        F: FnOnce() -> Fut + 'static,
        Fut: Future<Output = Result<Option<T>, anyhow::Error>> + 'static;

    #[allow(unused)]
    fn set_value<C: AppContext>(&self, cx: &mut C, value: T);

    fn set_idle<C: AppContext>(&self, cx: &mut C);
}

impl<T: 'static> AsyncResourceEntityExt<T> for Entity<AsyncResource<T>> {
    fn load<C, F, Fut>(&self, cx: &mut C, loader: F)
    where
        C: AppContext,
        F: FnOnce() -> Fut + 'static,
        Fut: Future<Output = Result<T, anyhow::Error>> + 'static,
    {
        AsyncResource::load(self, cx, loader)
    }

    fn maybe_load<C, F, Fut>(&self, cx: &mut C, loader: F)
    where
        C: AppContext,
        F: FnOnce() -> Fut + 'static,
        Fut: Future<Output = Result<Option<T>, anyhow::Error>> + 'static,
    {
        AsyncResource::maybe_load(self, cx, loader)
    }

    fn set_value<C: AppContext>(&self, cx: &mut C, value: T) {
        AsyncResource::set_value(self, cx, value)
    }

    fn set_idle<C: AppContext>(&self, cx: &mut C) {
        AsyncResource::set_idle(self, cx)
    }
}
