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

    /// Creates a new [`TaskPool`] which shares its capacity with this one.
    ///
    /// ```
    /// # use tokio_task_pool::TaskPool;
    /// # tokio_test::block_on(async {
    /// let mut task_pool: TaskPool<u8> = TaskPool::new(1);
    /// let mut sibling: TaskPool<&'static str> = task_pool.create_sibling();
    ///
    /// task_pool.spawn(async { 1 }).await;
    /// sibling.spawn(async { "2" }).await;
    ///
    /// assert_eq!(task_pool.join_next().await.unwrap().unwrap(), 1);
    /// assert_eq!(sibling.join_next().await.unwrap().unwrap(), "2");
    /// # })
    /// ```
    pub fn create_sibling<U: Send + 'static>(&self) -> TaskPool<U> {
        TaskPool::<U> {
            join_set: JoinSet::new(),
            semaphore: self.semaphore.clone(),
        }
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
