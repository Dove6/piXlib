use filesystem::DummyFileSystem;

use crate::{common::Position, runner::CallableIdentifier};

use super::*;

use test_case::test_case;

#[test_case("zero levels", "ABCDEFG", "ABCDEFG")]
#[test_case("one level", "\"ABCDEFG\"", "ABCDEFG")]
#[test_case("one level (left half)", "\"ABCDEFG", "ABCDEFG")]
#[test_case("one level (right half)", "ABCDEFG\"", "ABCDEFG")]
#[test_case("two levels", "\"\"ABCDEFG\"\"", "\"ABCDEFG\"")]
#[test_case("two levels (left half)", "\"\"ABCDEFG", "\"ABCDEFG")]
#[test_case("two levels (right half)", "ABCDEFG\"\"", "ABCDEFG\"")]
#[test_case("three levels", "\"\"\"ABCDEFG\"\"\"", "\"\"ABCDEFG\"\"")]
#[test_case("three levels (left half)", "\"\"\"ABCDEFG", "\"\"ABCDEFG")]
#[test_case("three levels (right half)", "ABCDEFG\"\"\"", "ABCDEFG\"\"")]
fn surrounding_quotes_should_be_handled_correctly_with_direct_set(
    _description: &str,
    argument: &str,
    expected: &str,
) {
    let runner = CnvRunner::try_new(
        Arc::new(RwLock::new(DummyFileSystem)),
        Default::default(),
        Default::default(),
    )
    .unwrap();
    let script = format!(
        r"
        OBJECT=TESTSTR
        TESTSTR:TYPE=STRING
        
        OBJECT=TESTBEH
        TESTBEH:TYPE=BEHAVIOUR
        TESTBEH:CODE={{TESTSTR^SET({});}}
        ",
        argument
    );
    runner
        .load_script(
            ScenePath::new(".", "SCRIPT.CNV"),
            as_parser_input(&script),
            None,
            ScriptSource::CnvLoader,
        )
        .unwrap();
    let test_beh_object = runner.get_object("TESTBEH").unwrap();
    test_beh_object
        .call_method(CallableIdentifier::Method("RUN"), &Vec::new(), None)
        .unwrap();
    let test_str_object = runner.get_object("TESTSTR").unwrap();
    let result = test_str_object
        .call_method(CallableIdentifier::Method("GET"), &Vec::new(), None)
        .unwrap();

    assert_eq!(result, CnvValue::String(expected.into()));
}

#[test_case("zero levels", "ABCDEFG", "HIJKLMN")]
#[test_case("one level", "\"ABCDEFG\"", "ABCDEFG")]
#[test_case("one level (left half)", "\"ABCDEFG", "ABCDEFG")]
#[test_case("one level (right half)", "ABCDEFG\"", "ABCDEFG")]
#[test_case("two levels", "\"\"ABCDEFG\"\"", "\"ABCDEFG\"")]
#[test_case("two levels (left half)", "\"\"ABCDEFG", "\"ABCDEFG")]
#[test_case("two levels (right half)", "ABCDEFG\"\"", "ABCDEFG\"")]
#[test_case("three levels", "\"\"\"ABCDEFG\"\"\"", "\"\"ABCDEFG\"\"")]
#[test_case("three levels (left half)", "\"\"\"ABCDEFG", "\"\"ABCDEFG")]
#[test_case("three levels (right half)", "ABCDEFG\"\"\"", "ABCDEFG\"\"")]
fn surrounding_quotes_should_be_handled_correctly_with_direct_set_and_inconveniently_named_variables(
    _description: &str,
    argument: &str,
    expected: &str,
) {
    let runner = CnvRunner::try_new(
        Arc::new(RwLock::new(DummyFileSystem)),
        Default::default(),
        Default::default(),
    )
    .unwrap();
    let script = format!(
        r"
        OBJECT=TESTSTR
        TESTSTR:TYPE=STRING

        OBJECT=ABCDEFG
        ABCDEFG:TYPE=STRING
        ABCDEFG:VALUE=HIJKLMN

        OBJECT=HIJKLMN
        HIJKLMN:TYPE=STRING
        HIJKLMN:VALUE=OPQRSTU

        OBJECT=OPQRSTU
        OPQRSTU:TYPE=STRING
        OPQRSTU:VALUE=VWXYZ
        
        OBJECT=TESTBEH
        TESTBEH:TYPE=BEHAVIOUR
        TESTBEH:CODE={{TESTSTR^SET({});}}
        ",
        argument
    );
    runner
        .load_script(
            ScenePath::new(".", "SCRIPT.CNV"),
            as_parser_input(&script),
            None,
            ScriptSource::CnvLoader,
        )
        .unwrap();
    let test_beh_object = runner.get_object("TESTBEH").unwrap();
    test_beh_object
        .call_method(CallableIdentifier::Method("RUN"), &Vec::new(), None)
        .unwrap();
    let test_str_object = runner.get_object("TESTSTR").unwrap();
    let result = test_str_object
        .call_method(CallableIdentifier::Method("GET"), &Vec::new(), None)
        .unwrap();

    assert_eq!(result, CnvValue::String(expected.into()));
}

#[test_case("zero levels", "ABCDEFG", "ABCDEFG")]
#[test_case("one level", "\"ABCDEFG\"", "ABCDEFG")]
#[test_case("one level (left half)", "\"ABCDEFG", "ABCDEFG")]
#[test_case("one level (right half)", "ABCDEFG\"", "ABCDEFG")]
#[test_case("two levels", "\"\"ABCDEFG\"\"", "ABCDEFG")]
#[test_case("two levels (left half)", "\"\"ABCDEFG", "ABCDEFG")]
#[test_case("two levels (right half)", "ABCDEFG\"\"", "ABCDEFG")]
#[test_case("three levels", "\"\"\"ABCDEFG\"\"\"", "\"ABCDEFG\"")]
#[test_case("three levels (left half)", "\"\"\"ABCDEFG", "\"ABCDEFG")]
#[test_case("three levels (right half)", "ABCDEFG\"\"\"", "ABCDEFG\"")]
fn surrounding_quotes_should_be_handled_correctly_with_one_level_indirect_set(
    _description: &str,
    argument: &str,
    expected: &str,
) {
    let runner = CnvRunner::try_new(
        Arc::new(RwLock::new(DummyFileSystem)),
        Default::default(),
        Default::default(),
    )
    .unwrap();
    let script = format!(
        r"
        OBJECT=TESTSTR
        TESTSTR:TYPE=STRING
        
        OBJECT=TESTBEH
        TESTBEH:TYPE=BEHAVIOUR
        TESTBEH:CODE={{TESTBEH2^RUN({});}}
        
        OBJECT=TESTBEH2
        TESTBEH2:TYPE=BEHAVIOUR
        TESTBEH2:CODE={{TESTSTR^SET($1);}}
        ",
        argument
    );
    runner
        .load_script(
            ScenePath::new(".", "SCRIPT.CNV"),
            as_parser_input(&script),
            None,
            ScriptSource::CnvLoader,
        )
        .unwrap();
    let test_beh_object = runner.get_object("TESTBEH").unwrap();
    test_beh_object
        .call_method(CallableIdentifier::Method("RUN"), &Vec::new(), None)
        .unwrap();
    let test_str_object = runner.get_object("TESTSTR").unwrap();
    let result = test_str_object
        .call_method(CallableIdentifier::Method("GET"), &Vec::new(), None)
        .unwrap();

    assert_eq!(result, CnvValue::String(expected.into()));
}

#[test_case("zero levels", "ABCDEFG", "VWXYZ")]
#[test_case("one level", "\"ABCDEFG\"", "OPQRSTU")]
#[test_case("one level (left half)", "\"ABCDEFG", "OPQRSTU")]
#[test_case("one level (right half)", "ABCDEFG\"", "OPQRSTU")]
#[test_case("two levels", "\"\"ABCDEFG\"\"", "HIJKLMN")]
#[test_case("two levels (left half)", "\"\"ABCDEFG", "HIJKLMN")]
#[test_case("two levels (right half)", "ABCDEFG\"\"", "HIJKLMN")]
#[test_case("three levels", "\"\"\"ABCDEFG\"\"\"", "ABCDEFG")]
#[test_case("three levels (left half)", "\"\"\"ABCDEFG", "ABCDEFG")]
#[test_case("three levels (right half)", "ABCDEFG\"\"\"", "ABCDEFG")]
fn surrounding_quotes_should_be_handled_correctly_with_two_level_indirect_set_and_existing_inconveniently_named_variables(
    _description: &str,
    argument: &str,
    expected: &str,
) {
    let runner = CnvRunner::try_new(
        Arc::new(RwLock::new(DummyFileSystem)),
        Default::default(),
        Default::default(),
    )
    .unwrap();
    let script = format!(
        r"
        OBJECT=TESTSTR
        TESTSTR:TYPE=STRING

        OBJECT=ABCDEFG
        ABCDEFG:TYPE=STRING
        ABCDEFG:VALUE=HIJKLMN

        OBJECT=HIJKLMN
        HIJKLMN:TYPE=STRING
        HIJKLMN:VALUE=OPQRSTU

        OBJECT=OPQRSTU
        OPQRSTU:TYPE=STRING
        OPQRSTU:VALUE=VWXYZ
        
        OBJECT=TESTBEH
        TESTBEH:TYPE=BEHAVIOUR
        TESTBEH:CODE={{TESTBEH2^RUN({});}}
        
        OBJECT=TESTBEH2
        TESTBEH2:TYPE=BEHAVIOUR
        TESTBEH2:CODE={{TESTBEH3^RUN($1);}}
        
        OBJECT=TESTBEH3
        TESTBEH3:TYPE=BEHAVIOUR
        TESTBEH3:CODE={{TESTSTR^SET($1);}}
        ",
        argument
    );
    runner
        .load_script(
            ScenePath::new(".", "SCRIPT.CNV"),
            as_parser_input(&script),
            None,
            ScriptSource::CnvLoader,
        )
        .unwrap();
    let test_beh_object = runner.get_object("TESTBEH").unwrap();
    test_beh_object
        .call_method(CallableIdentifier::Method("RUN"), &Vec::new(), None)
        .unwrap();
    let test_str_object = runner.get_object("TESTSTR").unwrap();
    let result = test_str_object
        .call_method(CallableIdentifier::Method("GET"), &Vec::new(), None)
        .unwrap();

    assert_eq!(result, CnvValue::String(expected.into()));
}

#[test]
fn behaviors_passed_by_name_should_handle_arguments_correctly() {
    let runner = CnvRunner::try_new(
        Arc::new(RwLock::new(DummyFileSystem)),
        Default::default(),
        Default::default(),
    )
    .unwrap();
    let script = r#"
        OBJECT=TESTSTR
        TESTSTR:TYPE=STRING
        TESTSTR:VALUE="ORIGINAL"
        TESTSTR:ONINIT=TESTBEH

        OBJECT=TESTBEH
        TESTBEH:TYPE=BEHAVIOUR
        TESTBEH:CODE={THIS^SET("TESTBEH");}
        "#;
    runner
        .load_script(
            ScenePath::new(".", "SCRIPT.CNV"),
            as_parser_input(script),
            None,
            ScriptSource::CnvLoader,
        )
        .unwrap();
    runner.step().unwrap();
    let test_str_object = runner.get_object("TESTSTR").unwrap();
    let result = test_str_object
        .call_method(CallableIdentifier::Method("GET"), &Vec::new(), None)
        .unwrap();

    assert_eq!(result, CnvValue::String("TESTBEH".into()));
}

fn as_parser_input(string: &str) -> impl Iterator<Item = declarative_parser::ParserInput> + '_ {
    string.chars().enumerate().map(|(i, c)| {
        Ok((
            Position {
                line: 1,
                column: 1 + i,
                character: i,
            },
            c,
            Position {
                line: 1,
                column: 2 + i,
                character: i + 1,
            },
        ))
    })
}
