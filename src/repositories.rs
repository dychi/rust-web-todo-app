use std::{
	collections::HashMap,
	sync::{Arc, RwLock, RwLockWriteGuard, RwLockReadGuard}
};
use anyhow::{Context, Ok};
use thiserror::Error;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Error)]
enum RepositoryError {
    #[error("NotFound, id is {0}")]
    NotFound(i32),
}

// リポジトリ
pub trait TodoRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    fn create(&self, payload: CreateTodo) -> Todo;
    fn find(&self, id: i32) -> Option<Todo>;
    fn all(&self) -> Vec<Todo>;
    fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo>;
    fn delete(&self, id: i32) -> anyhow::Result<()>;
}

// モデル
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Todo {
    id: i32,
    text: String,
    completed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Validate)]
pub struct CreateTodo {
	#[validate(length(min=1, message="Can not be empty"))]
	#[validate(length(max=100, message="Over text length"))]
    text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Validate)]
pub struct UpdateTodo {
	#[validate(length(min=1, message="Can not be empty"))]
	#[validate(length(max=100, message="Over text length"))]
    text: Option<String>,
    completed: Option<bool>,
}

impl Todo {
    pub fn new(id: i32, text: String) -> Self {
        Self {
            id,
            text,
            completed: false,
        }
    }
}

// datasource
type TodoDatas = HashMap<i32, Todo>;

#[derive(Debug, Clone)]
pub struct TodoRepositoryForMemory {
    store: Arc<RwLock<TodoDatas>>,
}

impl TodoRepositoryForMemory {
    pub fn new() -> Self {
        TodoRepositoryForMemory {
            store: Arc::default(),
        }
    }
	fn write_store_ref(&self) -> RwLockWriteGuard<TodoDatas> {
		self.store.write().unwrap()
	}
	fn read_store_ref(&self) -> RwLockReadGuard<TodoDatas> {
		self.store.read().unwrap()
	}
}

impl TodoRepository for TodoRepositoryForMemory {
    fn create(&self, payload: CreateTodo) -> Todo {
		let mut store = self.write_store_ref();
		let id = (store.len() + 1) as i32;
		let todo = Todo::new(id, payload.text.clone());
		store.insert(id, todo.clone());
		return todo
    }
    fn find(&self, id: i32) -> Option<Todo> {
		let store = self.read_store_ref();
		store.get(&id).map(|todo| todo.clone())
    }
    fn all(&self) -> Vec<Todo> {
		let store = self.read_store_ref();
		Vec::from_iter(store.values().map(|todo| todo.clone()))
    }
    fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo> {
		let mut store = self.write_store_ref();
		let todo = store
			.get(&id)
			.context(RepositoryError::NotFound(id))?;
		let text = payload.text.unwrap_or(todo.text.clone());
		let completed = payload.completed.unwrap_or(todo.completed);
		let todo = Todo {
			id,
			text,
			completed
		};
		store.insert(id, todo.clone());
		Ok(todo)
    }
    fn delete(&self, id: i32) -> anyhow::Result<()> {
		let mut store = self.write_store_ref();
		store.remove(&id).ok_or(RepositoryError::NotFound(id))?;
		Ok(())
    }
}


#[cfg(test)]
impl CreateTodo {
	pub fn new(text: String) -> Self {
		Self { text }
	}
}