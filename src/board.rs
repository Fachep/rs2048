use std::cell::{Cell, RefCell};
use std::fmt::{Display, Formatter};
use rand::prelude::*;
use crate::direction::Direction;
use crate::state::State;

type C = Cell<u8>;

pub struct Board<R> {
    cells: Vec<Vec<C>>,
    width: usize,
    height: usize,
    state: State,
    rand: RefCell<R>,
}

impl<R> Board<R>
    where R: Default
{
    pub fn new(width: usize, height: usize) -> Self {
        let cells = Vec::from_iter(
            (0..height).map(|_| {
                Vec::from_iter(
                    (0..width).map(|_| Default::default())
                )
            })
        );
        Board {
            cells,
            width,
            height,
            state: State::Uninitialized,
            rand: Default::default(),
        }
    }
}

impl Board<ThreadRng> {
    pub fn new_thread_rng(width: usize, height: usize) -> Self {
        let cells = Vec::from_iter(
            (0..height).map(|_| {
                Vec::from_iter(
                    (0..width).map(|_| Default::default())
                )
            })
        );
        Board {
            cells,
            width,
            height,
            state: State::Uninitialized,
            rand: Default::default(),
        }
    }
}

impl<R> Board<R>
    where R: SeedableRng
{
    pub fn new_with_seed(width: usize, height: usize, seed: R::Seed) -> Self {
        let cells = Vec::from_iter(
            (0..height).map(|_| {
                Vec::from_iter(
                    (0..width).map(|_| Default::default())
                )
            })
        );
        Board {
            cells,
            width,
            height,
            state: State::Uninitialized,
            rand: RefCell::new(R::from_seed(seed)),
        }
    }
}

impl<R> Board<R> {
    pub fn load(cells: Vec<Vec<u8>>, rng: R) -> Self {
        let height = cells.len();
        let width = cells.first().unwrap().len();
        let cells = cells.into_iter()
            .map(|line| {
                line.into_iter()
                    .map(|v| {
                        Cell::new(v)
                    })
                    .collect()
            })
            .collect();
        Board {
            cells,
            width,
            height,
            state: State::Stop,
            rand: RefCell::new(rng),
        }
    }
}

impl Default for Board<ThreadRng> {
    fn default() -> Self {
        Board::new_thread_rng(4, 4)
    }
}

impl<R> Display for Board<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.to_display().iter().try_for_each(|line| {
            writeln!(f, "+----+----+----+----+")?;
            line.iter().try_for_each(|cell| {
                write!(f, "|")?;
                match cell {
                    None => {
                        write!(f, "    ")
                    }
                    Some(v) => {
                        write!(f, "{:4}", v)
                    }
                }
            })?;
            writeln!(f, "|")
        })?;
        writeln!(f, "+----+----+----+----+")
    }
}

impl<R> Board<R> {
    fn to_display(&self) -> Vec<Vec<Option<u16>>> {
        self.cells.iter().map(|line| {
            line.iter().map(|cell| {
                if cell.get() == 0 {
                    None
                } else {
                    Some(1 << cell.get())
                }
            }).collect()
        }).collect()
    }
}

impl<R> Board<R>
    where R: Rng
{
    fn random_cell(&self) -> Option<&C> {
        self.cells.iter().flatten().filter(|c| c.get() == 0).choose(&mut *self.rand.borrow_mut())
    }

    fn try_generate(&self) -> bool {
        self.random_cell().map_or(false, |cell| {
            cell.set(1);
            true
        })
    }

    pub fn initialize(&mut self) {
        assert_eq!(self.state, State::Uninitialized);
        self.cells.iter()
            .flatten()
            .choose_multiple(&mut *self.rand.borrow_mut(), 2)
            .into_iter()
            .for_each(|c| {
                c.set(1)
            });
        self.state = State::Stop;
    }
}

impl<R> Board<R> {
    fn get_cells_rows(&self) -> Vec<Vec<&C>> {
        self.cells.iter()
            .map(|l| {
                l.iter().enumerate()
            })
            .flatten()
            .fold(
                Vec::from_iter((0..self.width).map(|_|Vec::new())),
                |mut rows, c| {
                    rows.get_mut(c.0).unwrap().push(c.1);
                    rows
                }
            )
    }

    fn cells_move<'c, It, I>(cells: I) -> bool
        where It: Iterator<Item = &'c C> + ExactSizeIterator + DoubleEndedIterator + Sized,
            I: IntoIterator<IntoIter = It> + Clone,
    {
        let it = cells.clone().into_iter();
        let len = it.len();
        let mut map = it.enumerate()
            .filter(|x| x.1.get().ne(&0))
            .map(|x| (x.0, x.1.get()));
        let mut result = false;
        let mut it = cells.into_iter();
        for i in 0..len {
            let c = it.next().unwrap();
            if let Some((j, v)) = map.next() {
                if i != j {
                    result = true;
                    c.set(v)
                }
            } else {
                c.set(0)
            }
        }
        result
    }

    fn try_move(&mut self) -> bool {
        let State::Step(ref direction) = self.state else { unreachable!() };
        match direction {
            Direction::Up => {
                let rows = self.get_cells_rows();
                rows.into_iter().map(Self::cells_move).fold(false, std::ops::BitOr::bitor)
            },
            Direction::Down => {
                let rows = self.get_cells_rows();
                rows.into_iter().map(|mut row| {
                    row.reverse();
                    Self::cells_move(row)
                }).fold(false, std::ops::BitOr::bitor)
            },
            Direction::Left => {
                self.cells.iter().map(Self::cells_move).fold(false, std::ops::BitOr::bitor)
            },
            Direction::Right => {
                self.cells.iter().map(|line| {
                    let line: Vec<_> = line.iter().rev().collect();
                    Self::cells_move(line)
                }).fold(false, std::ops::BitOr::bitor)
            },
        }
    }

    fn cells_merge<'c, It, I>(cells: I) -> bool
        where It: Iterator<Item = &'c C> + ExactSizeIterator + DoubleEndedIterator + Sized,
            I: IntoIterator<IntoIter = It> + Clone,
    {
        let mut it = cells.clone().into_iter().peekable();
        let mut result = false;
        while let (Some(c), Some(n)) = (it.next(), it.peek()) {
            if c.get().ne(&0) && c.get().eq(&n.get()) {
                c.set(c.get() + 1);
                n.set(0);
                result = true;
                it.next();
            }
        }
        result
    }

    fn try_merge(&mut self) -> bool {
        let State::Step(ref direction) = self.state else { unreachable!() };
        match direction {
            Direction::Up => {
                let rows = self.get_cells_rows();
                rows.into_iter().map(Self::cells_merge).fold(false, std::ops::BitOr::bitor)
            },
            Direction::Down => {
                let rows = self.get_cells_rows();
                rows.into_iter().map(|mut row| {
                    row.reverse();
                    Self::cells_merge(row)
                }).fold(false, std::ops::BitOr::bitor)
            },
            Direction::Left => {
                self.cells.iter().map(Self::cells_merge).fold(false, std::ops::BitOr::bitor)
            },
            Direction::Right => {
                self.cells.iter().map(|line| {
                    let line: Vec<_> = line.iter().rev().collect();
                    Self::cells_merge(line)
                }).fold(false, std::ops::BitOr::bitor)
            },
        }
    }
}

impl<R> Board<R>
    where R: Rng
{
    pub fn step(&mut self, direction: Direction) -> State {
        assert_eq!(self.state, State::Stop);
        if self.cells.iter().flatten().all(|c| c.get().ne(&0)) {
            self.state = State::Over(false);
            return self.state;
        }
        self.state = State::Step(direction);
        let mut result = false;
        let move_state = self.try_move();
        let merge_state = self.try_merge();
        if merge_state {
            self.try_move();
        }
        if merge_state || move_state {
            result = self.try_generate();
        }
        self.state = if result && self.cells.iter().flatten().any(|c| c.get() == 11) {
            State::Over(true)
        } else {
            State::Stop
        };
        self.state
    }

    pub fn state(&self) -> State {
        self.state
    }
}
