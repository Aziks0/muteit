// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

#![cfg(loom)]

use loom::thread;

use mpsc::channel;

#[test]
fn multiproducer() {
    loom::model(|| {
        let (tx, rx) = channel::<usize>(4);

        let tx1 = tx.clone();
        let t0 = thread::spawn(move || {
            tx1.try_send(2).unwrap();
        });
        let t1 = thread::spawn(move || {
            tx.try_send(4).unwrap();
        });

        t1.join().unwrap();
        t0.join().unwrap();

        let mut val = 0;
        val += rx.try_receive().unwrap();
        val += rx.try_receive().unwrap();

        assert_eq!(val, 6);
    });
}

#[test]
fn producer_consumer() {
    loom::model(|| {
        let (tx, rx) = channel::<usize>(4);

        let t = thread::spawn(move || {
            tx.try_send(1).unwrap();
            tx.try_send(2).unwrap();
            tx.try_send(3).unwrap();
        });

        let t = thread::spawn(move || {
            let mut val = 0;

            while let Some(v) = rx.try_receive() {
                val += v;
            }

            t.join().unwrap();

            while let Some(v) = rx.try_receive() {
                val += v;
            }

            assert_eq!(val, 6);
        });

        t.join().unwrap();
    });
}

#[test]
fn producer_consumer_full() {
    loom::model(|| {
        let (tx, rx) = channel::<usize>(3);

        // Make the channel full.
        tx.try_send(1).unwrap();
        tx.try_send(2).unwrap();
        tx.try_send(3).unwrap();

        let t0 = thread::spawn(move || tx.try_send(4));
        let t1 = thread::spawn(move || rx.try_receive().unwrap());

        let _ = t0.join().unwrap();
        t1.join().unwrap();
    });
}
