use crate::direction::Direction;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Uninitialized,
    Stop,
    Step(Direction),
    Win,
    Over,
}
