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
    let real: Vec<_> = (0..10000/2).map(|x| x*2).collect();

    let ids: Vec<_> = long.iter().collect();
    assert_eq!(ids, real);

    let mut x = 0;
    let long: IdSet = (1..1000).map(|n| { x += 2*n - 1; x }).collect();
    let real: Vec<_> = (1..1000).map(|n| n*n).collect();

    let ids: Vec<_> = long.into_iter().collect();
    assert_eq!(ids, real);
}

#[test]
fn filled() {
    for n in 0..100 {
        let set = IdSet::new_filled(n);
        for k in 0..n {
            assert!(set.contains(k));
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