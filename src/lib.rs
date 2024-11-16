use std::{future::Future, sync::Arc};

use tokio::{
    sync::Semaphore,
    task::{JoinError, JoinSet},
};

pub struct TaskPool<T> {
    join_set: JoinSet<T>,
    semaphore: Arc<Semaphore>,
}

impl<T: Send + 'static> TaskPool<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            join_set: JoinSet::new(),
            semaphore: Arc::new(Semaphore::new(capacity)),
        }
    }

    fn from_semaphore(semaphore: Arc<Semaphore>) -> Self {
        Self {
            join_set: JoinSet::new(),
            semaphore,
        }
    }

    /// Creates a new [`TaskPool`] which shares its capacity with this one.
    pub fn create_sibling(&self) -> Self {
        Self::from_semaphore(self.semaphore.clone())
    }

    /// A wrapper around [`JoinSet::spawn`] which acquires a permit from the internal semaphore.
    pub async fn spawn(&mut self, task: impl Future<Output = T> + Send + 'static) {
        let permit = self.semaphore.clone().acquire_owned().await.unwrap();
        self.join_set.spawn(async move {
            let result = task.await;
            drop(permit);
            result
        });
    }

    /// See [`JoinSet::join_next`].
    pub async fn join_next(&mut self) -> Option<Result<T, JoinError>> {
        self.join_set.join_next().await
    }
}
