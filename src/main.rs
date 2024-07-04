use std::io::{stdout, Write};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::{cursor, ExecutableCommand, QueueableCommand, terminal};
use crate::board::Board;
use crate::direction::Direction;
use crate::state::State;

mod board;
mod state;
mod direction;


fn main() -> std::io::Result<()> {
    let mut board = Board::default();
    board.initialize();
    let mut stdout = stdout();
    stdout.execute(terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    stdout.queue(cursor::Hide)?;
    stdout.queue(cursor::MoveTo(0, 0))?;
    writeln!(stdout, "{}", board)?;
    loop {
        let direction = match crossterm::event::read()? {
            Event::Key(KeyEvent {code, kind: KeyEventKind::Release, modifiers, .. }) => {
                match (code, modifiers) {
                    (KeyCode::Up, _) => Some(Direction::Up),
                    (KeyCode::Down, _) => Some(Direction::Down),
                    (KeyCode::Left, _) => Some(Direction::Left),
                    (KeyCode::Right, _) => Some(Direction::Right),
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                    _ => None
                }
            }
            _ => None
        };
        if let Some(direction) = direction {
            let state = board.step(direction);
            match state {
                State::Over(_) => {
                    break;
                }
                _ => {}
            }

            stdout.queue(terminal::Clear(terminal::ClearType::All))?;
            stdout.queue(cursor::MoveTo(0, 0))?;
            writeln!(stdout, "{}", board)?;
            stdout.flush()?;
        }
    }
    stdout.execute(cursor::Show)?;
    terminal::disable_raw_mode()?;
    stdout.execute(terminal::LeaveAlternateScreen)?;
    println!("{}", board);
    let state = board.state();
    if State::Over(true) == state {
        println!("You win!");
    } else if State::Over(false) == state {
        println!("Game over!");
    }
    Ok(())
}
