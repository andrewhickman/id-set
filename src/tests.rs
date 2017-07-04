use super::*;

#[test]
fn basic() {
    let mut b = IdSet::new();
    assert!(b.insert(3));
    assert!(!b.insert(3));
    assert!(b.contains(3));
    assert!(b.insert(4));
    assert!(!b.insert(4));
    assert!(b.contains(3));
    assert!(b.insert(400));
    assert!(!b.insert(400));
    assert!(b.contains(400));
    assert_eq!(b.len(), 3);
}

#[test]
fn remove() {
    let mut a = IdSet::new();

    assert!(a.insert(1));
    assert!(a.remove(1));

    assert!(a.insert(100));
    assert!(a.remove(100));

    assert!(a.insert(1000));
    assert!(a.remove(1000));

    assert_eq!(a.len(), 0);
}

#[test]
fn iterator() {
    let set: IdSet = vec![0, 2, 2, 3].into_iter().collect();

    let ids: Vec<Id> = set.iter().collect();
    assert_eq!(ids, [0, 2, 3]);

    let long: IdSet = (0..10000).filter(|&n| n % 2 == 0).collect();
    let real: Vec<_> = (0..10000 / 2).map(|x| x * 2).collect();

    let ids: Vec<_> = long.iter().collect();
    assert_eq!(ids, real);

    let mut x = 0;
    let long: IdSet = (1..1000)
        .map(|n| {
                 x += 2 * n - 1;
                 x
             })
        .collect();
    let real: Vec<_> = (1..1000).map(|n| n * n).collect();

    let ids: Vec<_> = long.into_iter().collect();
    assert_eq!(ids, real);
}

#[test]
fn filled() {
    for n in 0..100 {
        let set = IdSet::new_filled(n);
        for k in 0..n {
            assert!(set.contains(k), "not contained: {} < {}", k, n);
        }
        assert!(!set.contains(n));
    }
}

#[test]
fn clone() {
    let mut a = IdSet::new();

    assert!(a.insert(1));
    assert!(a.insert(100));
    assert!(a.insert(1000));

    let mut b = a.clone();

    assert!(a == b);

    assert!(b.remove(1));
    assert!(a.contains(1));

    assert!(a.remove(1000));
    assert!(b.contains(1000));
}

#[test]
fn eq() {
    let a = IdSet::from_bytes(&[0b10100010]);
    let b = IdSet::from_bytes(&[0b00000000]);
    let c = IdSet::new();

    assert!(a == a);
    assert!(a != b);
    assert!(a != c);
    assert!(b == b);
    assert!(b == c);
    assert!(c == c);
}

#[test]
fn from_bytes() {
    let a = IdSet::from_bytes(&[0b01101001]);

    assert!(a.contains(0));
    assert!(!a.contains(1));
    assert!(!a.contains(2));
    assert!(a.contains(3));
    assert!(!a.contains(4));
    assert!(a.contains(5));
    assert!(a.contains(6));
    assert!(!a.contains(7));
}

#[test]
fn retain() {
    let mut a = IdSet::new_filled(10);
    let b: IdSet = (0..5).map(|id| id * 2).collect();

    a.retain(|id| id % 2 == 0);

    assert_eq!(a, b);
}

#[test]
fn intersection() {
    let mut a = IdSet::new();
    let mut b = IdSet::new();

    assert!(a.insert(11));
    assert!(a.insert(1));
    assert!(a.insert(3));
    assert!(a.insert(77));
    assert!(a.insert(103));
    assert!(a.insert(5));

    assert!(b.insert(2));
    assert!(b.insert(11));
    assert!(b.insert(77));
    assert!(b.insert(5));
    assert!(b.insert(3));

    let expected = [3, 5, 11, 77];
    let actual: Vec<_> = a.intersection(&b).collect();
    assert_eq!(actual, expected);
}

#[test]
fn difference() {
    let mut a = IdSet::new();
    let mut b = IdSet::new();

    assert!(a.insert(1));
    assert!(a.insert(3));
    assert!(a.insert(5));
    assert!(a.insert(200));
    assert!(a.insert(500));

    assert!(b.insert(3));
    assert!(b.insert(200));

    let expected = [1, 5, 500];
    let actual: Vec<_> = a.difference(&b).collect();
    assert_eq!(actual, expected);
}

#[test]
fn symmetric_difference() {
    let mut a = IdSet::new();
    let mut b = IdSet::new();

    assert!(a.insert(1));
    assert!(a.insert(3));
    assert!(a.insert(5));
    assert!(a.insert(9));
    assert!(a.insert(11));

    assert!(b.insert(3));
    assert!(b.insert(9));
    assert!(b.insert(14));
    assert!(b.insert(220));

    let expected = [1, 5, 11, 14, 220];
    let actual: Vec<_> = a.symmetric_difference(&b).collect();
    assert_eq!(actual, expected);
}

#[test]
fn union() {
    let mut a = IdSet::new();
    let mut b = IdSet::new();
    assert!(a.insert(1));
    assert!(a.insert(3));
    assert!(a.insert(5));
    assert!(a.insert(9));
    assert!(a.insert(11));
    assert!(a.insert(160));
    assert!(a.insert(19));
    assert!(a.insert(24));
    assert!(a.insert(200));

    assert!(b.insert(1));
    assert!(b.insert(5));
    assert!(b.insert(9));
    assert!(b.insert(13));
    assert!(b.insert(19));

    let expected = [1, 3, 5, 9, 11, 13, 19, 24, 160, 200];
    let actual: Vec<_> = a.union(&b).collect();
    assert_eq!(actual, expected);
}

#[test]
fn subset() {
    let mut set1 = IdSet::new();
    let mut set2 = IdSet::new();

    assert!(set1.is_subset(&set2)); //  {}  {}
    set2.insert(100);
    assert!(set1.is_subset(&set2)); //  {}  { 1 }
    set2.insert(200);
    assert!(set1.is_subset(&set2)); //  {}  { 1, 2 }
    set1.insert(200);
    assert!(set1.is_subset(&set2)); //  { 2 }  { 1, 2 }
    set1.insert(300);
    assert!(!set1.is_subset(&set2)); // { 2, 3 }  { 1, 2 }
    set2.insert(300);
    assert!(set1.is_subset(&set2)); // { 2, 3 }  { 1, 2, 3 }
    set2.insert(400);
    assert!(set1.is_subset(&set2)); // { 2, 3 }  { 1, 2, 3, 4 }
    set2.remove(100);
    assert!(set1.is_subset(&set2)); // { 2, 3 }  { 2, 3, 4 }
    set2.remove(300);
    assert!(!set1.is_subset(&set2)); // { 2, 3 }  { 2, 4 }
    set1.remove(300);
    assert!(set1.is_subset(&set2)); // { 2 }  { 2, 4 }
}

#[test]
fn is_disjoint() {
    let a = IdSet::from_bytes(&[0b10100010]);
    let b = IdSet::from_bytes(&[0b01000000]);
    let c = IdSet::new();
    let d = IdSet::from_bytes(&[0b00110000]);

    assert!(!a.is_disjoint(&d));
    assert!(!d.is_disjoint(&a));

    assert!(a.is_disjoint(&b));
    assert!(a.is_disjoint(&c));
    assert!(b.is_disjoint(&a));
    assert!(b.is_disjoint(&c));
    assert!(c.is_disjoint(&a));
    assert!(c.is_disjoint(&b));
}

#[test]
fn inplace_union() {
    //a should grow to include larger elements
    let mut a = IdSet::new();
    a.insert(0);
    let mut b = IdSet::new();
    b.insert(5);
    let expected = IdSet::from_bytes(&[0b00100001]);
    a.inplace_union(&b);
    assert_eq!(a, expected);

    // Standard
    let mut a = IdSet::from_bytes(&[0b10100010]);
    let mut b = IdSet::from_bytes(&[0b01100010]);
    let c = a.clone();
    a.inplace_union(&b);
    b.inplace_union(&c);
    assert_eq!(a.len(), 4);
    assert_eq!(b.len(), 4);
}

#[test]
fn inplace_intersection() {
    // Explicitly 0'ed bits
    let mut a = IdSet::from_bytes(&[0b10100010]);
    let mut b = IdSet::from_bytes(&[0b00000000]);
    let c = a.clone();
    a.inplace_intersection(&b);
    b.inplace_intersection(&c);
    assert!(a.is_empty());
    assert!(b.is_empty());

    // Uninitialized bits should behave like 0's
    let mut a = IdSet::from_bytes(&[0b10100010]);
    let mut b = IdSet::new();
    let c = a.clone();
    a.inplace_intersection(&b);
    b.inplace_intersection(&c);
    assert!(a.is_empty());
    assert!(b.is_empty());

    // Standard
    let mut a = IdSet::from_bytes(&[0b10100010]);
    let mut b = IdSet::from_bytes(&[0b01100010]);
    let c = a.clone();
    a.inplace_intersection(&b);
    b.inplace_intersection(&c);
    assert_eq!(a.len(), 2);
    assert_eq!(b.len(), 2);
}

#[test]
fn inplace_difference() {
    // Explicitly 0'ed bits
    let mut a = IdSet::from_bytes(&[0b00000000]);
    let b = IdSet::from_bytes(&[0b10100010]);
    a.inplace_difference(&b);
    assert!(a.is_empty());

    // Uninitialized bits should behave like 0's
    let mut a = IdSet::new();
    let b = IdSet::from_bytes(&[0b11111111]);
    a.inplace_difference(&b);
    assert!(a.is_empty());

    // Standard
    let mut a = IdSet::from_bytes(&[0b10100010]);
    println!("{:?}", a);
    let mut b = IdSet::from_bytes(&[0b01100010]);
    println!("{:?}", b);
    let c = a.clone();
    a.inplace_difference(&b);
    println!("{:?}", a);
    b.inplace_difference(&c);
    println!("{:?}", b);
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
}

#[test]
fn inplace_symmetric_difference() {
    //a should grow to include larger elements
    let mut a = IdSet::new();
    a.insert(0);
    a.insert(1);
    let mut b = IdSet::new();
    b.insert(1);
    b.insert(5);
    let expected = IdSet::from_bytes(&[0b00100001]);
    a.inplace_symmetric_difference(&b);
    assert_eq!(a, expected);

    let mut a = IdSet::from_bytes(&[0b10100010]);
    let b = IdSet::new();
    let c = a.clone();
    a.inplace_symmetric_difference(&b);
    assert_eq!(a, c);

    // Standard
    let mut a = IdSet::from_bytes(&[0b11100010]);
    let mut b = IdSet::from_bytes(&[0b01101010]);
    let c = a.clone();
    a.inplace_symmetric_difference(&b);
    b.inplace_symmetric_difference(&c);
    assert_eq!(a.len(), 2);
    assert_eq!(b.len(), 2);
}

#[test]
fn block_iter_into_set() {
    let a: IdSet = (0..15).collect();
    let b: IdSet = (10..20).collect();
    let c: IdSet = (0..5).collect();

    let iter = (&a | &b) ^ c;

    let expected: IdSet = iter.clone().into_iter().collect();
    let actual: IdSet = iter.into_set();

    assert_eq!(expected, actual);
}