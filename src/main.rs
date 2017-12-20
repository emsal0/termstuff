extern crate termion;
extern crate tui;

use std::io;
use termion::event;
use termion::input::TermRead;

use std::thread;
use std::f64;

use std::iter::FromIterator;

use std::cmp::{max, min};

use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget, Block, border};
use tui::layout::{Group, Size, Rect, Direction};

struct AppState<'a> {

}

fn draw(t: &mut Terminal<TermionBackend>, n: &mut u16) {
    let size = t.size().unwrap();

    let p = 100 / *n;

    let block_sizes: Vec<tui::layout::Size> = (0..*n).map(|g| Size::Percent(p)).collect();

    Group::default()
        .direction(Direction::Vertical)
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

    terminal.clear().unwrap();
    draw(&mut terminal, &mut num_blocks);
    for c in stdin.keys() {

        let evt = c.unwrap();
        if evt == event::Key::Char('q') {
            break;
        } else if evt == event::Key::Char('a') {
            num_blocks = min(10, num_blocks + 1);
        } else if evt == event::Key::Char('r') {
            num_blocks = max(1, num_blocks - 1);
        }
        draw(&mut terminal, &mut num_blocks);
    }
}
