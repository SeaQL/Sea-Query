use super::*;
use pretty_assertions::assert_eq;

#[test]
fn signal_sqlstate() {
    let message = "Some error occurred";
    assert_eq!(
        ExceptionStatement::new(message.to_string()).to_string(MysqlQueryBuilder),
        format!("SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = '{message}'")
    );
}

#[test]
fn escapes_message() {
    let unescaped_message = "Does this 'break'?";
    assert_eq!(
        ExceptionStatement::new(unescaped_message.to_string()).to_string(MysqlQueryBuilder),
        format!("SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'Does this \\'break\\'?'")
    );
}
