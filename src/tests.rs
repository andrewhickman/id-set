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
fn test_bit_set_iterator() {
    let set: IdSet = vec![0, 2, 2, 3].into_iter().collect();

    let ids: Vec<Id> = set.iter().collect();
    assert_eq!(ids, [0, 2, 3]);

    let long: IdSet = (0..10000).filter(|&n| n % 2 == 0).collect();
    let real: Vec<_> = (0..10000/2).map(|x| x*2).collect();

    let idxs: Vec<_> = long.iter().collect();
    assert_eq!(idxs, real);
}