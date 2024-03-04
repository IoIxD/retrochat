use std::sync::Arc;

use futures::prelude::*;
use irc::client::prelude::*;
use parking_lot::Mutex;
use raylib::prelude::*;

pub struct MutexInnerMut<T>(Mutex<Vec<T>>)
where
    T: Clone;

impl<T> MutexInnerMut<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Self(Mutex::new(Vec::new()))
    }

    pub fn push(&self, val: T) {
        self.0.lock().push(val);
    }

    pub fn shift(&self) {
        let lock = &mut self.0.lock();
        lock.rotate_left(1);
        lock.pop();
    }

    pub fn len(&self) -> usize {
        self.0.lock().len()
    }

    pub fn inner(&self) -> Vec<T> {
        self.0.lock().clone()
    }
}

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    let (mut rl, thread) = raylib::init().size(640, 480).title("retrotwitch").build();

    let font = rl.get_font_default();

    let messages = Arc::new(MutexInnerMut::new());
    for _ in 0..100 {
        messages.push(String::new());
    }
    let m1 = messages.clone();
    let m2 = messages.clone();

    tokio::spawn(async move {
        let name = format!("#{}", std::env::args().last().unwrap());
        println!("{}", name);
        // We can also load the Config at runtime via Config::load("path/to/config.toml")
        let config = Config {
            nickname: Some("justinfan69".to_owned()),
            server: Some("irc.twitch.tv".to_owned()),
            channels: vec![name.to_owned()],
            port: Some(6667),
            use_tls: Some(false),

            ..Config::default()
        };

        let mut client = Client::from_config(config).await.unwrap();
        client.identify().unwrap();

        let mut stream = client.stream().unwrap();

        println!("connected!");

        while let Some(message) = stream.next().await.transpose().unwrap() {
            if let Some(m) = {
                if let Command::PRIVMSG(_auth, msg) = message.command {
                    if msg == "" {
                        continue;
                    }
                    if let Some(prefix) = message.prefix {
                        match prefix {
                            Prefix::ServerName(a) => Some(format!("{}: {}", a, msg)),
                            Prefix::Nickname(a, b, _) => {
                                if a == b {
                                    Some(format!("{}: {}", a, msg))
                                } else {
                                    Some(format!("{} ({}): {}", a, b, msg))
                                }
                            }
                        }
                    } else {
                        Some(format!("[???]: {}", msg))
                    }
                } else {
                    None
                }
            } {
                println!("{}", m);
                m1.push(m);
            }
        }
    });

    let mut starting_y = 480.0 - 32.0;

    while !rl.window_should_close() {
        starting_y += rl.get_mouse_wheel_move() * 16.0;

        if rl.is_key_released(KeyboardKey::KEY_F11) {
            rl.toggle_fullscreen();
        }
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);
        let mut m: Vec<String> = m2.inner();

        m.reverse();
        const MAX_Y: usize = 640;
        let mut y = starting_y as isize;
        let m = {
            if m.len() < MAX_Y {
                &m
            } else {
                &m[m.len() - MAX_Y..m.len() - 1]
            }
        }
        .to_vec();
        for msg in m {
            let height = {
                let len = msg.len() as isize;
                (((len / 2) + 31) & !31isize) as f32
            };
            let var_name = format!(" {}", msg);
            let rec = Rectangle::new(5.0, y as f32, 640.0, height);

            for i in 0..=1 {
                let rec = Rectangle::new(5.0 + i as f32, y as f32, 640.0, height);
                d.draw_rectangle(5, y as i32, 640, height as i32, Color::BLACK);
                d.draw_text_rec(
                    &font,
                    var_name.as_str(),
                    rec,
                    20.0,
                    3.0,
                    true,
                    Color::new(255, 191, 0, 255),
                );
                d.draw_rectangle_lines(
                    rec.x as i32,
                    rec.y as i32,
                    rec.width as i32,
                    rec.height as i32,
                    Color::BLACK,
                );
            }

            y -= (rec.height as isize);
        }
    }
    Ok(())
}
