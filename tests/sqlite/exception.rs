use super::*;
use pretty_assertions::assert_eq;

#[test]
fn select_raise_abort() {
    let message = "Some error occurred here";
    assert_eq!(
        ExceptionStatement::new(message.to_string()).to_string(SqliteQueryBuilder),
        format!("SELECT RAISE(ABORT, '{}')", message)
    );
}

#[test]
fn escapes_message() {
    let unescaped_message = "Does this 'break'?";
    let escaped_message = "Does this ''break''?";
    assert_eq!(
        ExceptionStatement::new(unescaped_message.to_string()).to_string(SqliteQueryBuilder),
        format!("SELECT RAISE(ABORT, '{}')", escaped_message)
    );
}
