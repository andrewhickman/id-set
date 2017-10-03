#![feature(test)]

extern crate test;
extern crate id_set;

use id_set::IdSet;

#[bench]
fn iter(b: &mut test::Bencher) {
    let set: IdSet = (0..10000).filter(|&n| n % 2 == 0).collect();

    b.iter(|| {
        for elem in &set {
            test::black_box(elem);
        }
    });
}

#[bench]
fn retain(b: &mut test::Bencher) {
    let set: IdSet = (0..10000).filter(|&n| n % 3 == 0).collect();

    b.iter(|| {
        // cloning is relatively fast compared to retain()
        set.clone().retain(|n| n % 2 == 0);
    });
}