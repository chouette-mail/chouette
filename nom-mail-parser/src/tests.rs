#[test]
fn parse_mail_1() {
    use crate::parse;
    assert!(parse(include_bytes!("../mails/simple.txt")).is_ok());
}

#[test]
fn parse_mail_2() {
    use crate::parse;
    assert!(parse(include_bytes!("../mails/big.txt")).is_ok());
}
