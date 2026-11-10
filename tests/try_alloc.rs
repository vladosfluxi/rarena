mod initialization {
    use rarena::arena::Arena;

    #[test]
    fn test_alloc_new() {
        let arena = Arena::new();
        println!("Arena capacity: {}", arena.get_capacity());
    }

    #[test]
    fn test_with_cap() {
        let arena = Arena::with_capacity(10 * 1024);
        println!("Arena capacity: {}", arena.get_capacity());
    }
}
