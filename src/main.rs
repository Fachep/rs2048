use std::io::{stdout, Write};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::{cursor, ExecutableCommand, QueueableCommand, terminal};
use rs2048::*;

fn main() -> std::io::Result<()> {
    let mut board = Board::default();
    board.initialize();
    let mut stdout = stdout();
    stdout.execute(terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    stdout.queue(cursor::Hide)?;
    while board.state() == State::Stop {
        stdout.queue(terminal::Clear(terminal::ClearType::All))?;
        stdout.queue(cursor::MoveTo(0, 0))?;
        writeln!(stdout, "{}", board)?;

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
            board.step(direction);
        }
    }
    stdout.execute(cursor::Show)?;
    terminal::disable_raw_mode()?;
    stdout.execute(terminal::LeaveAlternateScreen)?;
    println!("{}", board);
    match board.state() {
        State::Win => println!("You win!"),
        State::Over => println!("Game over!"),
        _ => ()
    }
    Ok(())
}
