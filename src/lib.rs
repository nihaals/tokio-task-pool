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

    pub async fn spawn(&mut self, task: impl Future<Output = T> + Send + 'static) {
        let permit = self.semaphore.clone().acquire_owned().await.unwrap();
        self.join_set.spawn(async move {
            let result = task.await;
            drop(permit);
            result
        });
    }

    pub async fn join_next(&mut self) -> Option<Result<T, JoinError>> {
        self.join_set.join_next().await
    }
}
