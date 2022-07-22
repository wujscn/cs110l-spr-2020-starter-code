use crossbeam_channel;
use std::{thread, time};

fn parallel_map<T, U, F>(mut input_vec: Vec<T>, num_threads: usize, f: F) -> Vec<U>
where
    F: FnOnce(T) -> U + Send + Copy + 'static,
    T: Send + 'static,
    U: Send + 'static + Default,
{
    let mut output_vec: Vec<U> = Vec::with_capacity(input_vec.len());
    output_vec.resize_with(input_vec.len(), Default::default);
    let (sender , receiver) = crossbeam_channel::unbounded::<(T, usize)>();
    let (rst_sender, rst_receiver) = crossbeam_channel::unbounded::<(U, usize)>();
    let mut threads = Vec::new();
    for _ in 0..num_threads {
        let receiver = receiver.clone();
        let rst_sender = rst_sender.clone();
        threads.push(thread::spawn(move || {
            while let Ok(next_item) = receiver.recv() {
                rst_sender.send((f(next_item.0), next_item.1))
                        .expect("Tried writing to channel, but there are no receivers!");
            }
        }));
    }
    let mut idx = input_vec.len();
    while let Some(item) = input_vec.pop() {
        idx -= 1;
        sender.send((item, idx))
            .expect("Tried writing to channel, but there are no receivers!");
    }
    drop(sender);
    drop(rst_sender);
    while let Ok((item, idx)) = rst_receiver.recv() {
        output_vec[idx] = item;
    }
    for thread in threads {
        thread.join().expect("Panic occurred in thread");
    }
    output_vec
}

fn main() {
    let v = vec![6, 7, 8, 9, 10, 1, 2, 3, 4, 5, 12, 18, 11, 5, 20];
    let squares = parallel_map(v, 10, |num| {
        println!("{} squared is {}", num, num * num);
        thread::sleep(time::Duration::from_millis(500));
        num * num
    });
    println!("squares: {:?}", squares);
}
