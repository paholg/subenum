use subenum::subenum;

#[subenum(SmallArray, LargeArray)]
#[derive(Debug, PartialEq, Clone)]
enum Buffer<T, const N: usize> {
    #[subenum(SmallArray)]
    Inline([T; N]),

    #[subenum(LargeArray)]
    Heap(Vec<T>),

    #[subenum(SmallArray, LargeArray)]
    Empty,
}

#[test]
fn test_const_generics_signature() {
    const SIZE: usize = 4;

    // 1. Test SmallArray preservation of <T, const N: usize>
    let small: SmallArray<u8, SIZE> = SmallArray::Inline([1, 2, 3, 4]);

    match small {
        SmallArray::Inline(arr) => {
            assert_eq!(arr.len(), SIZE);
            assert_eq!(arr[0], 1);
        }
        _ => panic!("Expected Inline variant"),
    }

    // 2. Test LargeArray preservation of <T>
    let large: LargeArray<u8> = LargeArray::Heap(vec![10, 20]);

    match large {
        LargeArray::Heap(vec) => assert_eq!(vec.len(), 2),
        _ => panic!("Expected Heap variant"),
    }

    // 3. Test shared variant
    let empty: SmallArray<u8, 10> = SmallArray::Empty;
    assert_eq!(empty, SmallArray::Empty);
}
