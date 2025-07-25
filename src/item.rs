use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Deserialize, Serialize, Clone, Debug, Copy)]
pub enum State {
    #[serde(rename = "stock")]
    Stock, // Means that the associated item is in stock
    #[serde(rename = "shopping")]
    Shopping, // Means that the associated item is in a shopping list
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let d = match self {
            State::Stock => "stock",
            State::Shopping => "shopping",
        };
        write!(f, "{d}")
    }
}

impl From<i64> for State {
    fn from(value: i64) -> Self {
        match value {
            2 => Self::Shopping,
            _ => Self::Stock,
        }
    }
}

pub struct Item {
    pub id: i64,
    pub name: String,
    pub quantity: f64,
    pub state: State,
}

impl Item {
    pub fn new(id: i64, name: String, quantity: f64, state: State) -> Self {
        Self {
            id,
            name,
            quantity,
            state,
        }
    }
}
