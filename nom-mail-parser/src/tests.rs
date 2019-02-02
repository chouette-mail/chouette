use crate::{parse, Result};

#[test]
fn parse_mail_1() -> Result<'static, ()> {
    let mail = parse(include_bytes!("../mails/simple.txt"))?;
    assert_eq!(mail.subject(), Some(&String::from("This is a test email")));
    Ok(())
}

#[test]
fn parse_mail_2() -> Result<'static, ()> {
    let mail = parse(include_bytes!("../mails/big.txt"))?;

    assert_eq!(
        mail.subject(),
        Some(&String::from("Aidez-nous à vous protéger\u{A0}: conseils de sécurité de Google")));

    Ok(())
}
