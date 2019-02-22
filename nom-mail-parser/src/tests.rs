use crate::{parse, Result};

#[test]
fn parse_mail_1() -> Result<()> {
    let mail = parse(include_bytes!("../mails/simple.txt"))?;
    assert_eq!(mail.subject(), Some(&String::from("This is a test email")));
    assert_eq!(mail.from(), Some(&String::from("Someone Something <someone@something.com>")));
    assert_eq!(mail.plain_body(), Some(&String::from("This is the plaintext version, in utf-8. Proof by Euro: =E2=82=AC\r\n")));
    assert_eq!(mail.html_body(), Some(&String::from("<html><body>This is the <b>HTML</b> version, in us-ascii. Proof by Euro: &euro;</body></html>\n")));
    Ok(())
}

#[test]
fn parse_mail_2() -> Result<()> {
    let mail = parse(include_bytes!("../mails/big.txt"))?;

    assert_eq!(
        mail.subject(),
        Some(&String::from("Aidez-nous à vous protéger\u{A0}: conseils de sécurité de Google")));

    Ok(())
}

#[test]
fn parse_mail_3() -> Result<()> {
    let mail = parse(include_bytes!("../mails/mail_validation.txt"))?;

    assert_eq!(
        mail.subject(),
        Some(&String::from("[SPAM] Welcome to Chouette Mail")));

    Ok(())
}

#[test]
fn parse_mail_4() -> Result<()> {
    let mail = parse(include_bytes!("../mails/mail_validation_2.txt"))?;

    assert_eq!(
        mail.subject(),
        Some(&String::from("Welcome to Chouette Mail")));

    Ok(())
}

