use crate::course::types::{Validation, ValidationMethod};
use crate::exec::environment::AssertionResult;
use crate::exec::sandbox::StepOutput;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ValidationResult {
    Success,
    OutputMismatch {
        expected: String,
        actual: String,
    },
    RegexMismatch {
        pattern: String,
        actual: String,
    },
    CompileSuccess,
    CustomScriptFailed {
        message: String,
    },
    StateAssertionFailed {
        results: Vec<AssertionResult>,
    },
}

impl ValidationResult {
    pub fn is_success(&self) -> bool {
        matches!(self, ValidationResult::Success | ValidationResult::CompileSuccess)
    }
}

pub fn validate_output(validation: &Validation, output: &StepOutput) -> ValidationResult {
    match validation.method {
        ValidationMethod::Output => {
            let expected = validation
                .expected_output
                .as_deref()
                .unwrap_or("")
                .trim();
            let actual = output.stdout.trim();

            if actual == expected {
                ValidationResult::Success
            } else {
                ValidationResult::OutputMismatch {
                    expected: expected.to_string(),
                    actual: actual.to_string(),
                }
            }
        }
        ValidationMethod::CompileOnly => {
            // If we got here, compilation succeeded (check_exit_code would have stopped us)
            ValidationResult::CompileSuccess
        }
        ValidationMethod::Regex => {
            let pattern = validation.pattern.as_deref().unwrap_or("");
            let actual = output.stdout.trim();

            match regex::Regex::new(pattern) {
                Ok(re) => {
                    if re.is_match(actual) {
                        ValidationResult::Success
                    } else {
                        ValidationResult::RegexMismatch {
                            pattern: pattern.to_string(),
                            actual: actual.to_string(),
                        }
                    }
                }
                Err(e) => ValidationResult::CustomScriptFailed {
                    message: format!("Invalid regex '{}': {}", pattern, e),
                },
            }
        }
        ValidationMethod::Custom => {
            // Custom validation via script — not implemented in Phase 1 MVP
            ValidationResult::CustomScriptFailed {
                message: "Custom validation not yet implemented".to_string(),
            }
        }
        ValidationMethod::State => {
            // State validation is handled in the runner, not here.
            // If we reach this, it means no assertions were configured.
            ValidationResult::Success
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_match_exact() {
        let validation = Validation {
            method: ValidationMethod::Output,
            expected_output: Some("42".to_string()),
            pattern: None,
            script: None,
            assertions: None,
        };
        let output = StepOutput {
            stdout: "42\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
            timed_out: false,
        };
        assert!(validate_output(&validation, &output).is_success());
    }

    #[test]
    fn test_output_mismatch() {
        let validation = Validation {
            method: ValidationMethod::Output,
            expected_output: Some("42".to_string()),
            pattern: None,
            script: None,
            assertions: None,
        };
        let output = StepOutput {
            stdout: "43\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
            timed_out: false,
        };
        assert!(!validate_output(&validation, &output).is_success());
    }

    #[test]
    fn test_regex_match() {
        let validation = Validation {
            method: ValidationMethod::Regex,
            expected_output: None,
            pattern: Some(r"^\d+$".to_string()),
            script: None,
            assertions: None,
        };
        let output = StepOutput {
            stdout: "42\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
            timed_out: false,
        };
        assert!(validate_output(&validation, &output).is_success());
    }

    #[test]
    fn test_compile_only() {
        let validation = Validation {
            method: ValidationMethod::CompileOnly,
            expected_output: None,
            pattern: None,
            script: None,
            assertions: None,
        };
        let output = StepOutput::default();
        assert!(validate_output(&validation, &output).is_success());
    }
}
