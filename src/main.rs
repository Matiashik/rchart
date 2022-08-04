use ncurses as nc;
use std::{env, fs, io::Write, sync::mpsc, thread, time};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    match args.len() {
        3 => prebuild(
            String::from(args[1].as_str()),
            time::Duration::from_millis(args[2].parse::<u64>().unwrap_or(100)),
        ),
        2 => prebuild(
            String::from(args[1].as_str()),
            time::Duration::from_millis(100),
        ),
        _ => prebuild(String::from(""), time::Duration::from_millis(100)),
    }
}

fn prebuild(path: String, tick: time::Duration) {
    let (sx, rx) = mpsc::channel();
    thread::spawn(move || {
        let rnd = ['/', '—', '\\', '|'];
        let l = rnd.len();
        let mut c: usize = 0;

        while rx
            .recv_timeout(time::Duration::from_micros(0))
            .unwrap_or(true)
        {
            print!("⏳ Loading file {}\r", rnd[c % l]);
            c += 1;
            std::io::stdout().flush().unwrap();
            thread::sleep(time::Duration::from_millis(100));
        }
    });
    println!("{}", path);
    let mut r = match fs::read(path.as_str()) {
        Result::Ok(res) => res,
        Result::Err(err) => {
            println!("❌ Error during reading a file");
            println!("📁 Reaing itself");
            match fs::read(env::current_exe().unwrap().as_os_str().to_str().unwrap()) {
                Result::Ok(res) => res,
                Result::Err(err) => {
                    println!("❌ Error during reading a file");
                    panic!("{}", err.to_string());
                }
            }
        }
    }
    .iter()
    .map(|x| {
        let mut s = format!("{:b}", *x);
        for _ in 0..(8 - s.len()) {
            s = String::from("0") + s.as_str();
        }
        s
    })
    .collect::<String>();

    sx.send(false).unwrap();
    println!("✅ File successfully loaded");

    graphics(&mut r, path, tick);
}

fn graphics(r: &mut String, path: String, tick: time::Duration) {
    nc::initscr();
    nc::start_color();
    nc::refresh();
    nc::curs_set(nc::CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    nc::border(0, 0, 0, 0, 0, 0, 0, 0);
    nc::refresh();

    let LINES = nc::getmaxy(nc::stdscr()) as u16;
    let COLUMNS = nc::getmaxx(nc::stdscr()) as u16;
    nc::init_pair(1, nc::COLOR_BLACK, nc::COLOR_GREEN);
    nc::init_pair(2, nc::COLOR_BLACK, nc::COLOR_RED);
    nc::init_pair(3, nc::COLOR_WHITE, nc::COLOR_BLACK);
    nc::init_pair(4, nc::COLOR_BLACK, nc::COLOR_MAGENTA);
    nc::init_pair(5, nc::COLOR_BLACK, nc::COLOR_BLUE);
    nc::init_pair(6, nc::COLOR_BLACK, nc::COLOR_YELLOW);
    nc::init_pair(7, nc::COLOR_MAGENTA, nc::COLOR_BLACK);

    let w = nc::newwin(LINES as i32, 5, 0, 0);
    nc::wborder(
        w,
        0,
        0,
        0,
        0,
        nc::ACS_HLINE(),
        nc::ACS_BSSS(),
        nc::ACS_HLINE(),
        nc::ACS_BTEE(),
    );
    nc::wrefresh(w);
    nc::refresh();

    let mut hist: Vec<i128> = Vec::new();
    let mut diff: i128 = 0;
    let mut cent = diff;
    let mut up = if r.starts_with('1') { true } else { false };

    while let car = r.chars().next().unwrap_or('$') {
        r.remove(0);
        match car {
            '1' => diff += 1,
            '0' => diff -= 1,
            _ => break,
        }
        hist.push(diff);
        if hist.len() > (COLUMNS - 6) as usize {
            up = hist[1] > hist[0];
            hist.remove(0);
        }
        nc::color_set(3);
        for h in 1..LINES - 1 {
            for w in 5..COLUMNS - 1 {
                nc::mv(h as i32, w as i32);
                nc::addch(' ' as u32);
            }
            nc::mv(h as i32, 4);
            nc::addch(nc::ACS_VLINE());
        }
        nc::mv(0, 5);
        nc::addstr({
            match format!("Current diff is: {}; ^C to exit", diff) {
                x if x.len() <= (COLUMNS - 5) as usize => x,
                _ => match format!("Current diff is: {}", diff) {
                    x if x.len() <= (COLUMNS - 5) as usize => x,
                    _ => match format!("{}", diff) {
                        x if x.len() <= (COLUMNS - 5) as usize => x,
                        _ => String::from(""),
                    },
                },
            }
            .as_str()
        });

        if LINES % 2 == 0 {
            let ost: i16 = (LINES as i16) / 2 - 1;
            if diff > cent + ost as i128 {
                cent += 1;
            } else if diff < cent - ost as i128 + 1 {
                cent -= 1;
            }
            nc::color_set(6);
            for i in (0 - ost)..ost {
                nc::color_set(if i == 0 {
                    4
                } else if (cent - i as i128) >= 0 {
                    6
                } else {
                    5
                });
                nc::mv((1 + i + ost) as i32, 0);
                nc::addstr((beautify(cent - i as i128)).as_str());
            }

            nc::mv(ost as i32 + 1, 4);
            nc::color_set(7);
            nc::addch(nc::ACS_LTEE());
            for i in 0..(COLUMNS - 5 - 1) {
                nc::addch(nc::ACS_HLINE());
            }
            nc::addch(nc::ACS_RTEE());

            for i in 0..hist.len() {
                let val = hist[i];
                if val > cent + ost as i128 || val <= cent - ost as i128 {
                    continue;
                } 
                let c: char;
                if i == 0 {
                    nc::color_set(if up { 1 } else { 2 });
                } else {
                    nc::color_set(if hist[i] > hist[i - 1] { 1 } else { 2 });
                }
                nc::mv((ost + (cent - val) as i16 + 1) as i32, 5 + (i as i32));
                nc::addch(nc::ACS_BLOCK());
                if cent - val != 0 {
                    nc::color_set(3);
                    nc::mv((ost + (cent - val) as i16 + 1) as i32, 4);
                    nc::addch(nc::ACS_VLINE());
                } else {
                    nc::color_set(7);
                    nc::mv((ost + (cent - val) as i16 + 1) as i32, 4);
                    nc::addch(nc::ACS_LTEE());
                }
            }
        } //todo: else

        nc::wrefresh(nc::stdscr());
        nc::color_set(3);
        nc::refresh();
        thread::sleep(tick);
    }

    nc::getch();
    nc::endwin();
}

fn beautify(_d: i128) -> String {
    if format!("{}", _d.abs()).len() <= 3 {
        return format!(
            "{}{}",
            match format!("{}", _d.abs()).len() {
                1 => "   ",
                2 => "  ",
                _ => " ",
            },
            _d.abs()
        );
    }
    let mut d = _d.abs();
    let mut r: String = String::from("");
    while format!("{}", d).len() > 3 {
        d = d / 1000;
        r = (String::from("K") + r.as_str())
            .replace("KK", "M")
            .replace("KM", "B")
            .replace("KB", "T")
            .replace("KT", "Q");
    }
    return format!(
        "{}{}{}",
        match format!("{}", d).len() {
            1 => "  ",
            2 => " ",
            _ => "",
        },
        d,
        r
    );
}
