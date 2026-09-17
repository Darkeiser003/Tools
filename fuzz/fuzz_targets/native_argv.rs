#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };
    if let Ok(arguments) = ltools_argv::split_native_argument_line(input) {
        assert!(arguments.len() <= 63);
        assert!(arguments.iter().all(|argument| argument.len() <= 4_096));
        assert!(arguments
            .iter()
            .all(|argument| !argument.chars().any(char::is_control)));
    }
});
