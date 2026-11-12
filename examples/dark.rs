#![no_std]
#![no_main]

use core::ffi::CStr;
use kos::{
    graphics::{display_message, Color, Dot, Size},
    input::fetch_key,
    threads::{exit, fetch_event, Event},
    windows::{
        define_button, define_window, end_window_draw, get_button_id, invert_pixel,
        start_window_draw, WindowKind, WindowParams, CLOSE_BUTTON,
    },
};

const HEADER: &'static CStr = c"Dark Mode Demo";
const TEXT: [&'static CStr; 6] = [
    c"Lorem ipsum dolor sit amet,",
    c"semper et rutrum placerat,",
    c"Integer sed diam commodo quam varius",
    c"Sed finibus urna sit amet felis",
    c"vestibulum elementum. Maecenas at feugiat lacus",
    c"tristique et sit amet tortor.",
];
const BTN: u32 = 42;
const WINDOW_SIZE: Size = Size {
    width: 400,
    height: 400,
};

extern crate alloc;

fn draw_window(invert: bool) {
    start_window_draw();

    define_window(
        Dot { x: 50, y: 50 },
        WINDOW_SIZE,
        WindowParams {
            color: Color::rgb(0xff, 0xff, 0xff),
            kind: WindowKind::FixedThemed,
            title: Some(HEADER),
        },
    );

    display_message(Dot { x: 10, y: 10 }, Color::rgb(0, 0, 0), TEXT[0], None);
    display_message(Dot { x: 10, y: 50 }, Color::rgb(0, 0, 0), TEXT[1], None);
    display_message(Dot { x: 10, y: 90 }, Color::rgb(0, 0, 0), TEXT[2], None);
    display_message(Dot { x: 10, y: 130 }, Color::rgb(0, 0, 0), TEXT[3], None);
    display_message(Dot { x: 10, y: 170 }, Color::rgb(0, 0, 0), TEXT[4], None);
    display_message(Dot { x: 10, y: 210 }, Color::rgb(0, 0, 0), TEXT[5], None);

    define_button(
        Dot { x: 10, y: 300 },
        Size {
            width: 100,
            height: 30,
        },
        BTN,
        true,
        true,
        Some(Color::rgb(147, 112, 219)),
    );

    display_message(
        Dot { x: 20, y: 310 },
        Color::rgb(255, 255, 255),
        if invert {
            c"Light mode"
        } else {
            c"Dark mode"
        },
        None,
    );

    if invert {
        for x in 0..WINDOW_SIZE.width {
            for y in 0..WINDOW_SIZE.height {
                invert_pixel(Dot { x, y })
            }
        }
    }

    end_window_draw();

    return;
}

fn button_handler(invert: &mut bool) {
    let btn_id = get_button_id();

    if btn_id.is_some() {
        match btn_id.unwrap() {
            CLOSE_BUTTON => exit(),
            BTN => {
                *invert = !*invert;
                draw_window(*invert);
            }
            _ => {}
        }
    }
}

#[no_mangle]
fn kol_main() {
    let mut invert = false;

    while let Some(ev) =
        fetch_event((Event::Redraw as u32) | (Event::KeyPress as u32) | (Event::BtnPress as u32))
    {
        match ev {
            Event::Redraw => draw_window(invert),
            Event::KeyPress => drop(fetch_key()),
            Event::BtnPress => button_handler(&mut invert),
            _ => break,
        }
    }
}
