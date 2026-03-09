// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

#![cfg(not(loom))]

use mpsc::channel;

#[test]
fn send() {
    let (tx, _) = channel::<usize>(5);
    assert_eq!(tx.try_send(0), Ok(()));
}

#[test]
fn receive() {
    let s = String::from("test");

    let (tx, rx) = channel::<_>(5);
    let _ = tx.try_send(s.clone());

    assert_eq!(rx.try_receive(), Some(s));
    assert_eq!(rx.try_receive(), None);
}

#[test]
fn stop_sending_when_full() {
    const CAPACITY: usize = 4;
    let (tx, _) = channel::<usize>(CAPACITY);

    for _ in 0..CAPACITY {
        assert_eq!(tx.try_send(0), Ok(()));
    }

    assert_eq!(tx.try_send(10), Err(10));
}
