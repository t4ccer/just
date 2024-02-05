// Adapted from https://gitlab.freedesktop.org/xorg/app/xlsfonts/-/blob/master/xlsfonts.c
// Copyright 1989, 1998  The Open Group

// Equivalent to running `xlsfonts -l`

use justshow::x11::{error::Error, requests, XDisplay};

fn go() -> Result<(), Error> {
    let mut display = XDisplay::open()?;

    let request = requests::ListFontsWithInfo {
        max_names: u16::MAX,
        pattern: b"*".to_vec(),
    };

    let pending_reply = display.send_request(&request)?;
    display.flush()?;
    let mut reply = display.await_pending_reply(pending_reply)?;
    reply.replies.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));

    println!("DIR  MIN  MAX EXIST DFLT PROP ASC DESC NAME");
    for piece in reply.replies {
        if let Ok(name) = std::str::from_utf8(&piece.name) {
            print!(
                "{}",
                match piece.draw_direction {
                    justshow::x11::replies::DrawDirection::LeftToRight => "--> ",
                    justshow::x11::replies::DrawDirection::RightToLeft => "<-- ",
                }
            );

            if piece.min_byte1 == 0 && piece.max_byte1 == 0 {
                print!(" {:>3} ", piece.min_char_or_byte2);
                print!(" {:>3} ", piece.max_char_or_byte2);
            } else {
                print!("*{:>3} ", piece.min_char_or_byte2);
                print!("*{:>3} ", piece.max_char_or_byte2);
            }

            print!("{:>5} ", if piece.all_chars_exist { "all" } else { "some" });
            print!("{:>4} ", piece.default_char);
            print!("{:>4} ", piece.properties.len());
            print!("{:>3} ", piece.font_ascent);
            print!("{:>4} ", piece.font_descent);
            println!("{}", name);
        } else {
            eprintln!("Could not parse font name to utf8: '{:?}'", piece.name);
        }
    }

    Ok(())
}

fn main() {
    match go() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("xlsfonts: error: {}", err);
        }
    }
}
