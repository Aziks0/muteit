// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

#![cfg(not(loom))]

use std::thread;
use std::time::Duration;

use futures_executor::block_on;

use mpsc::channel;

#[test]
fn producer_consumer_full_contended() {
    let (tx, rx) = channel::<usize>(2);

    // Make the channel full.
    tx.try_send(1).unwrap();
    tx.try_send(2).unwrap();

    let tx1 = tx.clone();
    let t0 = thread::spawn(move || {
        let _ = tx1.try_send(3);
        let _ = tx1.try_send(4);
        let _ = tx1.try_send(5);
        let _ = tx1.try_send(6);
    });
    let t1 = thread::spawn(move || {
        let _ = tx.try_send(7);
        let _ = tx.try_send(8);
        let _ = tx.try_send(9);
        let _ = tx.try_send(10);
    });

    while let Some(_) = rx.try_receive() {}

    let _ = t0.join().unwrap();
    t1.join().unwrap();

    while let Some(_) = rx.try_receive() {}
}

#[test]
fn async_receive() {
    let (tx, rx) = channel::<usize>(4);

    let t = thread::spawn(move || {
        tx.try_send(1).unwrap();
        tx.try_send(2).unwrap();

        thread::sleep(Duration::from_millis(200));

        tx.try_send(3).unwrap();
        tx.try_send(4).unwrap();
    });

    let mut val = 0;
    for _ in 0..4 {
        val += block_on(rx.receive());
    }

    let _ = t.join().unwrap();

    assert_eq!(rx.try_receive(), None);
    assert_eq!(val, 10);
}

#[test]
fn async_stress() {
    const THREADS: usize = 4;
    const VALUES: usize = if cfg!(miri) { 1_000 } else { 100_000 };

    let (tx, rx) = channel::<usize>(4);

    let t_senders = [0; THREADS].map(|_| {
        let tx = tx.clone();
        thread::spawn(move || {
            for i in 0..VALUES {
                while let Err(_) = tx.try_send(i) {
                    for _ in 0..5 {
                        std::hint::spin_loop();
                    }
                }
            }
        })
    });

    let t_receiver = thread::spawn(move || {
        for _ in 0..VALUES * THREADS {
            block_on(rx.receive());
        }

        assert_eq!(rx.try_receive(), None);
    });

    let _ = t_senders.map(|t| t.join().unwrap());
    t_receiver.join().unwrap();
}
