use std::path::PathBuf;
use nonvolatile::State;
use crate::error::Error;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

#[derive(Serialize, Deserialize)]
pub struct PersistedProject {
    pub id: i32,
    pub path: PathBuf,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct PersistedState {
  pub theme: Theme,
  pub projects: Vec<PersistedProject>,
  pub user_name: String,
}

pub struct PersistedStateManager {
  nonvolatile_state: State
}

impl PersistedStateManager {
  pub fn new() -> Result<Self, Error> {
    Ok(PersistedStateManager {
      nonvolatile_state: State::load_else_create("splatcad-state")?
    })
  }

  pub fn get_all(&self) -> Result<PersistedState, Error> {
    // self.nonvolatile_state.
    // Ok(self.nonvolatile_state.get().ok_or(Error::PersistedStateError("State not found".to_string()))?)
    todo!()
  }

  pub fn set_theme(self, theme: Theme) {
    // self.nonvolatile_state.set("theme", theme);
    todo!()
  }
}