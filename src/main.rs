extern crate chan;
extern crate termion;
extern crate tui;
extern crate rand;

use std::io;
use termion::event;
use termion::input::TermRead;

use std::sync::mpsc;
use std::thread;
use std::sync::{Arc, Mutex};

use std::time;
use std::iter::FromIterator;

use std::cmp::{max, min};

use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget, Block, border};
use tui::layout::{Group, Size, Rect, Direction};

struct AppState {
    numWindows: u16,
    direction: tui::layout::Direction,
}

fn draw(t: &mut Terminal<TermionBackend>, state: &mut AppState) {
    let size = t.size().unwrap();

    let stateVal = &*state;

    let p = 100 / stateVal.numWindows;

    let block_sizes: Vec<tui::layout::Size> = (0..stateVal.numWindows).map(|g| Size::Percent(p)).collect();

    Group::default()
        .direction(state.direction.clone())
        .margin(1)
        .sizes(&block_sizes)
        .render(t, &size, |t, chunks| {
            for chunk in chunks {
                Block::default()
                    .title("Block")
                    .borders(border::ALL)
                    .render(t, chunk);
            }
        });

    t.draw().unwrap();
}

fn main() {

    let stdin = io::stdin();

    let backend = TermionBackend::new().unwrap();

    let mut terminal = Terminal::new(backend).unwrap();
    let mut num_blocks = 1;
    let mut appState = AppState {
        numWindows: 1,
        direction: Direction::Vertical,
    };

    let terminal = Arc::new(Mutex::new(terminal));
    let appState = Arc::new(Mutex::new(appState));

    let state = Arc::clone(&appState);
    let term = Arc::clone(&terminal);

    let (tx, rx) = chan::sync(2);

    term.lock().unwrap().clear().unwrap();
    draw(&mut term.lock().unwrap(), &mut state.lock().unwrap());

    let rx2 = rx.clone();
    let directionSwitcherThread = thread::spawn(move || {
        loop {
            let val = rx2.recv().unwrap();
            if (val) {
                break;
            }
            thread::sleep(time::Duration::from_secs(1));
            let mut lState = state.lock().unwrap();
            let mut lTerm = term.lock().unwrap();
            let newDirection = match lState.direction {
                Direction::Vertical => Direction::Horizontal,
                Direction::Horizontal => Direction::Vertical,
            };
            lState.direction = newDirection;
            draw(&mut lTerm, &mut lState);
            thread::yield_now();
        }
    });

    let state2 = Arc::clone(&appState);
    let term2 = Arc::clone(&terminal);

    let addWindowThread = thread::spawn(move || {
        let tx = tx.clone();

        for c in stdin.keys() {
            let mut eState = state2.lock().unwrap();
            let mut eTerm = term2.lock().unwrap();

            let evt = c.unwrap();
            if evt == event::Key::Char('q') {
                tx.send(true);
                break;
            } else if evt == event::Key::Char('a') {
                eState.numWindows = min(10, eState.numWindows + 1);
            } else if evt == event::Key::Char('r') {
                eState.numWindows = max(1, eState.numWindows - 1);
            }
            tx.send(false);
            draw(&mut eTerm, &mut eState);
            thread::yield_now();
        }
    });

    loop {
        let val = rx.recv().unwrap();
        if (val) {
            break;
        }
    }
}
