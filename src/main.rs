use futures::channel::mpsc::{self};
use futures::executor::{self, ThreadPool, ThreadPoolBuilder};
use futures::stream::StreamExt;
use rand::{rngs::OsRng, RngCore};
use std::time::Instant;

//use std::mpsc::{UnboundedReceiver, UnboundedSender};
use std::time;

pub struct PwdImprover {
    threads: ThreadPool,
    rng: OsRng,
}

impl PwdImprover {
    pub fn new(n: usize) -> Self {
        let threads = ThreadPoolBuilder::new().pool_size(n).create().unwrap();
        let rng = OsRng {};
        PwdImprover { threads, rng }
    }

    pub async fn improve(&self, password: &str, additional_length: usize) -> String {
        let mut range = self.rng;
        let (tx, mut rx) = mpsc::unbounded::<String>();

        let char_set: Vec<char> = format!("0123456789!&#_@*~$={}", password).chars().collect();

        let mut result: Vec<char> = vec!['0'; additional_length + password.len()];
        let insert_index = range.next_u32() as usize % (char_set.len() - password.len());

        for elem in result.iter_mut().take(insert_index) {
            *elem = char_set[range.next_u32() as usize % char_set.len()];
        }
        let mut pwd_iter = password.chars();
        for elem in result.iter_mut().skip(insert_index).take(password.len()) {
            *elem = pwd_iter.next().unwrap();
        }
        for elem in result.iter_mut().skip(insert_index + password.len()) {
            *elem = char_set[range.next_u32() as usize % char_set.len()];
        }

        let fut_tx = async move {
            let sleep = time::Duration::from_secs(rand::random::<u64>() % 3 + 1);
            std::thread::sleep(sleep);
            tx.unbounded_send(result.iter().collect())
                .expect("send failed");
        };

        self.threads.spawn_ok(fut_tx);

        rx.next().await.unwrap()
    }
}

fn main() {
    let improver = PwdImprover::new(4);
    let now = Instant::now();
    let improved = executor::block_on(improver.improve("hello", 32));
    println!("Elapsed={:?} for {}", now.elapsed().as_millis(), improved);
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::future::join_all;

    #[test]
    fn test_password_improver() {
        let bad_passwords = vec![
            "hello".to_string(),
            "badPassword".to_string(),
            "veryShort".to_string(),
            "my name".to_string(),
            "my cat's name".to_string(),
        ];
        let improver = PwdImprover::new(4);
        let mut futures = vec![];
        let now = Instant::now();
        for password in bad_passwords.iter() {
            futures.push(improver.improve(password, 32));
        }
        let results = executor::block_on(join_all(futures));
        println!("Elapsed={:?}", now.elapsed());
        for (password, result) in bad_passwords.iter().zip(results.iter()) {
            if !result.contains(password) {
                panic!("{} does not contain {}", result, password)
            } else {
                println!("{} improved to {}", password, result);
            };
        }
    }
}
