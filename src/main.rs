use rand::{rngs::OsRng, RngCore};
use std::{sync::mpsc, thread, time};

//static mut RNG: OsRng = OsRng {};

struct PasswordImprover {
    ch: mpsc::SyncSender<String>,
}

impl PasswordImprover {
    fn new(ch: mpsc::SyncSender<String>) -> Self {
        Self { ch }
    }

    fn improve(&self, val: String, length: usize) {
        const MIN_SLEEP: u64 = 1;
        const MAX_SLEEP: u64 = 3;
        let mut RNG: OsRng = OsRng {};

        let char_set = format!("0123456789!&#_@*~$={}", val);
        let mut result: Vec<u8> = vec![0; length + val.len()];
        let insert_index = RNG.next_u32() as usize % (char_set.len() - val.len());

        for (idx, elem) in result.iter_mut().enumerate() {
            *elem = char_set.as_bytes()[RNG.next_u32() as usize % char_set.len()];
            //.get(RNG.next_u32() as usize % char_set.len())
            //.unwrap()
            //.clone();

            if idx >= insert_index && idx < insert_index + val.len() {
                *elem = val.as_bytes()[idx - insert_index];
                //*elem = *val.as_bytes().get(idx - insert_index).unwrap();
            }
            //println!("{} {}", idx, (*elem) as char);
        }
        let ch = self.ch.clone();

        // pretend this is doing some work or syscalls or something
        thread::spawn(move || {
            let sleep = time::Duration::from_secs(
                rand::random::<u64>() % (MAX_SLEEP - MIN_SLEEP) + MIN_SLEEP,
            );
            std::thread::sleep(sleep);

            ch.send(String::from_utf8_lossy(&result).to_string())
                .unwrap();
        });
    }
}

fn main() {
    test_password_improver()
}

fn test_password_improver() {
    let (tx, rx) = mpsc::sync_channel(1);
    let improver = PasswordImprover { ch: tx };

    let bad_passwords = vec![
        "hello".to_string(),
        "badPassword".to_string(),
        "veryShort".to_string(),
    ];

    for password in bad_passwords {
        improver.improve(password.clone(), 32);
        match rx.recv() {
            Ok(pw) => {
                if !pw.contains(&password) {
                    panic!("{} does not contain {}", pw, password)
                } else {
                    println!("{} improved to {}", password, pw);
                };
            }
            Err(_) => return,
        }
    }
}
