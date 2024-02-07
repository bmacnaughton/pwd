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

    // password - the user supplied password
    // length - the length of the bytes to add to the password
    fn improve(&self, password: &str, length: usize) {
        const MIN_SLEEP: u64 = 1;
        const MAX_SLEEP: u64 = 3;
        let mut range: OsRng = OsRng {};

        let char_set = format!("0123456789!&#_@*~$={}", password);
        let mut result: Vec<u8> = vec![0; length + password.len()];
        // where to insert the user-supplied password into the result
        let insert_index = range.next_u32() as usize % (char_set.len() - password.len());

        // this is technically not correct as you just can't treat chars as bytes. either
        // result needs to be Vec<char> or password and char_set need to be Vec<u8> (or &[u8]).
        for elem in result.iter_mut().take(insert_index) {
            *elem = char_set.as_bytes()[range.next_u32() as usize % char_set.len()];
        }

        for (ix, elem) in result
            .iter_mut()
            .skip(insert_index)
            .take(password.len())
            .enumerate()
        {
            *elem = password.as_bytes()[ix];
        }

        for elem in result.iter_mut().skip(insert_index + password.len()) {
            *elem = char_set.as_bytes()[range.next_u32() as usize % char_set.len()];
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
        improver.improve(&password, 32);
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
