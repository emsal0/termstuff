#[macro_use]
extern crate chan;
extern crate termion;
extern crate tui;
extern crate rand;

use std::io;
use termion::event;
use termion::input::TermRead;

use std::thread;
use std::sync::{Arc, Mutex};

use std::time;

use std::cmp::{max, min};

use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget, Block, border};
use tui::layout::{Group, Size, Direction};

struct AppState {
    num_windows: u16,
    direction: tui::layout::Direction,
}

fn draw(t: &mut Terminal<TermionBackend>, state: &mut AppState) {
    let size = t.size().unwrap();

    let state_val = &*state;

    let p = 100 / state_val.num_windows;

    let block_sizes: Vec<tui::layout::Size> = (0..state_val.num_windows).map(|_| Size::Percent(p)).collect();

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
    let mut app_state = AppState {
        num_windows: 1,
        direction: Direction::Vertical,
    };

    let terminal = Arc::new(Mutex::new(terminal));
    let app_state = Arc::new(Mutex::new(app_state));

    let state = Arc::clone(&app_state);
    let term = Arc::clone(&terminal);

    let (tx, rx) = chan::sync(0);

    term.lock().unwrap().clear().unwrap();
    draw(&mut term.lock().unwrap(), &mut state.lock().unwrap());

    let rx2 = rx.clone();
    thread::spawn(move || {
        loop {
            chan_select! {
                default => {
                    thread::sleep(time::Duration::from_secs(1));
                    let mut lstate = state.lock().unwrap();
                    let mut lterm = term.lock().unwrap();
                    let new_direction = match lstate.direction {
                        Direction::Vertical => Direction::Horizontal,
                        Direction::Horizontal => Direction::Vertical,
                    };
                    lstate.direction = new_direction;
                    draw(&mut lterm, &mut lstate);
                    thread::yield_now();
                },
                rx2.recv() => {
                    return;
                },
            }
        }
    });

    let state = Arc::clone(&app_state);
    let term = Arc::clone(&terminal);

    thread::spawn(move || {
        let tx = tx.clone();

        for c in stdin.keys() {
            let mut estate = state.lock().unwrap();
            let mut eterm = term.lock().unwrap();

            let evt = c.unwrap();
            if evt == event::Key::Char('q') {
                tx.send(true);
                break;
            } else if evt == event::Key::Char('a') {
                estate.num_windows = min(10, estate.num_windows + 1);
            } else if evt == event::Key::Char('r') {
                estate.num_windows = max(1, estate.num_windows - 1);
            }
            draw(&mut eterm, &mut estate);
            thread::yield_now();
        }
    });

    loop {
        let val = rx.recv().unwrap();
        if (val) {
            break;
        }
    }
    let term = Arc::clone(&terminal);
    let mut t = term.lock().unwrap();
    t.show_cursor().unwrap();
    t.clear().unwrap();
}
