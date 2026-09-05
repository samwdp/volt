use super::{ProgramLocation, locate_program};

#[test]
fn missing_program_is_missing() {
    assert_eq!(
        locate_program("volt-definitely-not-a-real-program-xyz"),
        ProgramLocation::Missing
    );
}
