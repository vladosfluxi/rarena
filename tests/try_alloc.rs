mod initialization {
    use rarena::arena::Arena;

    #[test]
    fn test_init() {
        let arena = Arena::new();
        println!("Arena capacity: {}", arena.get_capacity());
    }

    #[test]
    fn test_with_cap() {
        let arena = Arena::with_capacity(10 * 1024);
        println!("Arena capacity: {}", arena.get_capacity());
    }
    #[test]
    fn test_default_new() {
        let mut arena = Arena::new();
        let num = arena.alloc(42_i32).unwrap();
        assert_eq!(*num, 42);

        *num = 100;
        assert_eq!(*num, 100);
    }

    #[test]
    fn test_multiple_types_and_alignment() {
        let mut arena = Arena::with_capacity(1024);

        let k = arena.alloc([1, 2, 3, 4, 5]);
        let b = arena.alloc(255_u8).unwrap();
        let i = arena.alloc(123456_u64).unwrap();
        let f = arena.alloc(3.14159_f32).unwrap();
        let s = arena.alloc(42_u16).unwrap();

        assert_eq!(*b, 255);
        assert_eq!(*i, 123456);
        assert_eq!(*f, 3.14159);
        assert_eq!(*s, 42);
    }

    #[test]
    fn test_complex_structs() {
        #[derive(Debug, PartialEq)]
        struct Point {
            x: i32,
            y: i32,
        }

        let mut arena = Arena::with_capacity(1024);

        let p1 = arena
            .alloc(Point {
                x: 10,
                y: 20,
            })
            .unwrap();
        let p2 = arena
            .alloc(Point {
                x: -5,
                y: 15,
            })
            .unwrap();

        assert_eq!(
            *p1,
            Point {
                x: 10,
                y: 20
            }
        );
        assert_eq!(
            *p2,
            Point {
                x: -5,
                y: 15
            }
        );

        p1.x = 99;
        assert_eq!(p1.x, 99);
    }

    #[test]
    fn test_out_of_memory_error() {
        let mut arena = Arena::with_capacity(4);

        let result = arena.alloc(9999_u64);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not enough memory");
    }
    #[test]
    fn test_helpers() {
        let arena = Arena::with_capacity(8);
        let result = arena.alloc(0_i64);

        assert_eq!(arena.allocated_bytes(), 8);
        assert_eq!(arena.remaining_capacity(), 0);
    }

    #[test]
    fn test_alloc_slice_uninitialized() {
        let arena = Arena::with_capacity(1024);

        let slice: &mut [i32] = arena.alloc_slice(4).unwrap();

        for (i, val) in slice.iter_mut().enumerate() {
            *val = (i as i32 + 1) * 10;
        }

        assert_eq!(slice, &[10, 20, 30, 40]);
    }
}
