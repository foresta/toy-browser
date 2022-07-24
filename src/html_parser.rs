mod test {
    use super::*;

    #[test]
    fn test_parse_attriute() {
        assert_eq!(
            attribute().easy_parse("test=\"foobar\""),
            Ok((("test".to_string(), "foobar".to_string()), ""))
        )
    }
}
