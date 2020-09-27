#![feature(test)]

extern crate test;

use test::{Bencher, black_box};

const WORK_FACTOR: u32 = 54000;
#[bench]
fn pow2_via_mod(b: &mut Bencher) {
    for _ in 0..WORK_FACTOR {
        for i in 0..=u16::MAX {
            black_box(is_mod(i));
        }
    }
}

#[bench]
fn pow2_via_floor_ceil(b: &mut Bencher) {
    for _ in 0..WORK_FACTOR {
        for i in 0..=u16::MAX {
            black_box(is_floor_ceil(i));
        }
    }
}

fn is_floor_ceil(v: u16) -> bool {
    let flt: f32 = v.into();
    let log = flt.log2();
    log.ceil() == log.floor()
}

fn is_mod(v: u16) -> bool {
    let flt: f32 = v.into();
    flt.log2() % 1.0 == 0.0
}