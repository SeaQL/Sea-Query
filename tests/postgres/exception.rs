use super::*;
use pretty_assertions::assert_eq;

#[test]
fn raise_exception() {
    let message = "Some error occurred";
    assert_eq!(
        ExceptionStatement::new(message.to_string()).to_string(PostgresQueryBuilder),
        format!("RAISE EXCEPTION '{message}'")
    );
}

#[test]
fn escapes_message() {
    let unescaped_message = "Does this 'break'?";
    assert_eq!(
        ExceptionStatement::new(unescaped_message.to_string()).to_string(PostgresQueryBuilder),
        format!("RAISE EXCEPTION E'Does this \\'break\\'?'")
    );
}
