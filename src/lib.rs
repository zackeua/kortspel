//! Shared card types and game engines for the Kortspel collection.

pub mod cards;
pub mod games;
pub mod simulator;

#[cfg(feature = "graphics")]
pub mod card_renderer;

pub use cards::{Card, Rank, Suit};
pub use games::vandtia::{Game, MoveError, PlayerPhase, TurnOutcome};
