use binary_options_tools_core::{traits::Rule, Rule};

struct TestRuleImpl;

impl TestRuleImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for TestRuleImpl {
    fn call(&self, msg: &tokio_tungstenite::tungstenite::Message) -> bool {
        true
    }

    fn reset(&self) {}
}

// ============================================================================
// SIMPLE MATCHER TESTS
// ============================================================================

#[Rule]
#[rule({any()})]
struct SimpleAny;

#[Rule]
#[rule({never()})]
struct SimpleNever;

#[Rule]
#[rule({exact("test")})]
struct SimpleExact;

#[Rule]
#[rule({starts_with("prefix")})]
struct SimpleStartsWith;

#[Rule]
#[rule({ends_with("suffix")})]
struct SimpleEndsWith;

#[Rule]
#[rule({contains("middle")})]
struct SimpleContains;

#[Rule]
#[rule({regex("^[0-9]+$")})]
struct SimpleRegex;

// ============================================================================
// BINARY MATCHER TESTS
// ============================================================================

#[Rule]
#[rule({binary_exact([0x01, 0x02, 0x03])})]
struct BinaryExactRule;

#[Rule]
#[rule({binary_starts_with([0xFF, 0xFE])})]
struct BinaryStartsWithRule;

#[Rule]
#[rule({binary_ends_with([0x00, 0x01])})]
struct BinaryEndsWithRule;

#[Rule]
#[rule({binary_contains([0xAB, 0xCD])})]
struct BinaryContainsRule;

// ============================================================================
// METHOD CHAIN TESTS
// ============================================================================
// TODO: Fix chained method parsing - currently has issues with argument parsing
/*
#[Rule]
#[rule({starts_with("prefix").wait(1)})]
struct ChainedWait;

#[Rule]
#[rule({starts_with("prefix").wait(5).wait_messages(10)})]
struct ChainedMultipleMethods;

#[Rule]
#[rule({contains("test").lstrip_then("prefix")})]
struct ChainedLstripThen;

#[Rule]
#[rule({contains("test").rstrip_then("suffix")})]
struct ChainedRstripThen;

#[Rule]
#[rule({contains("test").lstrip_until(":")})]
struct ChainedLstripUntil;

#[Rule]
#[rule({contains("test").rstrip_until(";")})]
struct ChainedRstripUntil;
*/

// ============================================================================
// AND OPERATOR TESTS
// ============================================================================

#[Rule]
#[rule({starts_with("a") & ends_with("b")})]
struct AndTwoOperands;

#[Rule]
#[rule({starts_with("a") & contains("b") & ends_with("c")})]
struct AndThreeOperands;

#[Rule]
#[rule({any() & any() & any() & any()})]
struct AndMultipleAny;

#[Rule]
#[rule({starts_with("test").wait(1) & contains("data").wait(2)})]
struct AndWithChainedMethods;

// ============================================================================
// OR OPERATOR TESTS
// ============================================================================

#[Rule]
#[rule({starts_with("a") | ends_with("b")})]
struct OrTwoOperands;

#[Rule]
#[rule({starts_with("a") | contains("b") | ends_with("c")})]
struct OrThreeOperands;

#[Rule]
#[rule({never() | never() | any()})]
struct OrMultipleOperands;

#[Rule]
#[rule({exact("x").wait(1) | exact("y").wait(2)})]
struct OrWithChainedMethods;

// ============================================================================
// THEN (SEQUENCE) OPERATOR TESTS
// ============================================================================

#[Rule]
#[rule({starts_with("a") -> ends_with("b")})]
struct ThenTwoOperands;

#[Rule]
#[rule({starts_with("a") -> contains("b") -> ends_with("c")})]
struct ThenThreeOperands;

#[Rule]
#[rule({any() -> any()})]
struct ThenWithAny;

#[Rule]
#[rule({starts_with("test").wait(1) -> contains("data")})]
struct ThenWithChainedMethods;

// ============================================================================
// NOT OPERATOR TESTS
// ============================================================================

#[Rule]
#[rule({!any()})]
struct NotSimple;

#[Rule]
#[rule({!starts_with("test")})]
struct NotStartsWith;

#[Rule]
#[rule({!contains("error").wait(1)})]
struct NotWithChainedMethods;

// ============================================================================
// MIXED OPERATORS - AND WITH NOT
// ============================================================================

#[Rule]
#[rule({starts_with("test") & !contains("error")})]
struct AndWithNot;

#[Rule]
#[rule({!starts_with("a") & !ends_with("b")})]
struct AndWithMultipleNot;

#[Rule]
#[rule({starts_with("ok") & !contains("error").wait(1) & ends_with("!")})]
struct AndWithNotAndChained;

// ============================================================================
// MIXED OPERATORS - OR WITH NOT
// ============================================================================

#[Rule]
#[rule({starts_with("a") | !ends_with("b")})]
struct OrWithNot;

#[Rule]
#[rule({!exact("error") | !exact("fail")})]
struct OrWithMultipleNot;

#[Rule]
#[rule({!contains("bad").wait(1) | contains("good")})]
struct OrWithNotAndChained;

// ============================================================================
// PRECEDENCE TESTS - PARENTHESES
// ============================================================================

#[Rule]
#[rule({(starts_with("a") & ends_with("b")) | contains("c")})]
struct PrecedenceAndOr;

#[Rule]
#[rule({starts_with("a") & (ends_with("b") | contains("c"))})]
struct PrecedenceOrAnd;

#[Rule]
#[rule({(starts_with("a") -> ends_with("b")) | contains("c")})]
struct PrecedenceThenOr;

#[Rule]
#[rule({!(starts_with("a") & ends_with("b"))})]
struct PrecedenceNotAnd;

#[Rule]
#[rule({!(starts_with("a") | ends_with("b"))})]
struct PrecedenceNotOr;

// ============================================================================
// DEEP NESTING TESTS
// ============================================================================

#[Rule]
#[rule({((starts_with("a") & ends_with("b")) | (contains("c") & exact("d")))})]
struct DeeplyNestedAndOr;

#[Rule]
#[rule({(starts_with("a") & (ends_with("b") | (contains("c") & exact("d"))))})]
struct DeeplyNestedMixed;

#[Rule]
#[rule({!(!starts_with("a"))})]
struct DoubleNegation;

#[Rule]
#[rule({((starts_with("a") -> ends_with("b")) | (contains("c") -> exact("d")))})]
struct DeeplyNestedThen;

// ============================================================================
// COMPLEX COMBINATIONS
// ============================================================================

#[Rule]
#[rule({starts_with("42") & !contains("error")})]
struct OriginalTestCase;

#[Rule]
#[rule({(starts_with("status") & contains("200")) | (starts_with("error") & !contains("timeout"))})]
struct ComplexHttpStatus;

#[Rule]
#[rule({starts_with("msg") -> (contains("data") & !contains("null")) -> ends_with("!")})]
struct ComplexSequence;

#[Rule]
#[rule({(regex("^[0-9]+$") | regex("^[a-z]+$")) & !contains("invalid")})]
struct ComplexRegexAndOr;

#[Rule]
#[rule({(starts_with("a") | starts_with("b")) & (ends_with("x") | ends_with("y"))})]
struct ComplexWithMultipleChains;

// ============================================================================
// BINARY MATCHER COMBINATIONS
// ============================================================================

#[Rule]
#[rule({binary_exact([0x01]) | binary_contains([0xFF])})]
struct BinaryOr;

#[Rule]
#[rule({binary_starts_with([0x00]) & binary_ends_with([0xFF])})]
struct BinaryAnd;

#[Rule]
#[rule({!binary_contains([0xBA, 0xD0])})]
struct BinaryNot;

// ============================================================================
// MIXED TEXT AND BINARY
// ============================================================================

#[Rule]
#[rule({starts_with("text") & binary_contains([0xFF])})]
struct TextBinaryAnd;

#[Rule]
#[rule({contains("msg") | binary_exact([0x42, 0x42])})]
struct TextBinaryOr;

// ============================================================================
// MESSAGE TYPE MATCHER TESTS (if applicable)
// ============================================================================

// Note: MessageType would need to be properly imported from the core crate
// Uncomment when available:
/*
#[Rule]
#[rule({message_type(Text)})]
struct MessageTypeText;

#[Rule]
#[rule({message_type(Text) & starts_with("test")})]
struct MessageTypeWithMatcher;
*/

// ============================================================================
// CUSTOM MATCHER TESTS
// ============================================================================

#[Rule]
#[rule({custom(|_msg| { true })})]
struct CustomSimple;

#[Rule]
#[rule({custom(|_msg| { true }) & starts_with("test")})]
struct CustomWithAnd;

#[Rule]
#[rule({custom(|_msg| { true }) | ends_with("!")})]
struct CustomWithOr;

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[Rule]
#[rule({any()})]
struct EdgeCaseAnyAlone;

#[Rule]
#[rule({never()})]
struct EdgeCaseNeverAlone;

#[Rule]
#[rule({!any()})]
struct EdgeCaseNotAny;

#[Rule]
#[rule({any() -> never()})]
struct EdgeCaseAnyThenNever;

#[Rule]
#[rule({(any())})]
struct EdgeCaseParenthesizedAny;

#[Rule]
#[rule({((((any()))))})]
struct EdgeCaseMultipleParens;

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_any_compiles() {
        let _rule = SimpleAny::new();
    }

    #[test]
    fn test_simple_never_compiles() {
        let _rule = SimpleNever::new();
    }

    #[test]
    fn test_simple_exact_compiles() {
        let _rule = SimpleExact::new();
    }

    #[test]
    fn test_simple_starts_with_compiles() {
        let _rule = SimpleStartsWith::new();
    }

    #[test]
    fn test_and_two_operands_compiles() {
        let _rule = AndTwoOperands::new();
    }

    #[test]
    fn test_and_three_operands_compiles() {
        let _rule = AndThreeOperands::new();
    }

    #[test]
    fn test_or_two_operands_compiles() {
        let _rule = OrTwoOperands::new();
    }

    #[test]
    fn test_or_three_operands_compiles() {
        let _rule = OrThreeOperands::new();
    }

    #[test]
    fn test_then_two_operands_compiles() {
        let _rule = ThenTwoOperands::new();
    }

    #[test]
    fn test_then_three_operands_compiles() {
        let _rule = ThenThreeOperands::new();
    }

    #[test]
    fn test_not_simple_compiles() {
        let _rule = NotSimple::new();
    }

    #[test]
    fn test_not_starts_with_compiles() {
        let _rule = NotStartsWith::new();
    }

    #[test]
    fn test_and_with_not_compiles() {
        let _rule = AndWithNot::new();
    }

    #[test]
    fn test_or_with_not_compiles() {
        let _rule = OrWithNot::new();
    }

    #[test]
    fn test_precedence_and_or_compiles() {
        let _rule = PrecedenceAndOr::new();
    }

    #[test]
    fn test_precedence_or_and_compiles() {
        let _rule = PrecedenceOrAnd::new();
    }

    #[test]
    fn test_precedence_not_and_compiles() {
        let _rule = PrecedenceNotAnd::new();
    }

    #[test]
    fn test_deeply_nested_and_or_compiles() {
        let _rule = DeeplyNestedAndOr::new();
    }

    #[test]
    fn test_deeply_nested_mixed_compiles() {
        let _rule = DeeplyNestedMixed::new();
    }

    #[test]
    fn test_double_negation_compiles() {
        let _rule = DoubleNegation::new();
    }

    #[test]
    fn test_original_test_case_compiles() {
        let _rule = OriginalTestCase::new();
    }

    #[test]
    fn test_complex_http_status_compiles() {
        let _rule = ComplexHttpStatus::new();
    }

    #[test]
    fn test_complex_sequence_compiles() {
        let _rule = ComplexSequence::new();
    }

    #[test]
    fn test_complex_regex_and_or_compiles() {
        let _rule = ComplexRegexAndOr::new();
    }

    #[test]
    fn test_complex_with_multiple_chains_compiles() {
        let _rule = ComplexWithMultipleChains::new();
    }

    #[test]
    fn test_binary_or_compiles() {
        let _rule = BinaryOr::new();
    }

    #[test]
    fn test_binary_and_compiles() {
        let _rule = BinaryAnd::new();
    }

    #[test]
    fn test_binary_not_compiles() {
        let _rule = BinaryNot::new();
    }

    #[test]
    fn test_text_binary_and_compiles() {
        let _rule = TextBinaryAnd::new();
    }

    #[test]
    fn test_text_binary_or_compiles() {
        let _rule = TextBinaryOr::new();
    }

    #[test]
    fn test_custom_simple_compiles() {
        let _rule = CustomSimple::new();
    }

    #[test]
    fn test_custom_with_and_compiles() {
        let _rule = CustomWithAnd::new();
    }

    #[test]
    fn test_custom_with_or_compiles() {
        let _rule = CustomWithOr::new();
    }

    // TODO: Fix chained method tests
    /*
    #[test]
    fn test_chained_wait_compiles() {
        let _rule = ChainedWait::new();
    }

    #[test]
    fn test_chained_multiple_methods_compiles() {
        let _rule = ChainedMultipleMethods::new();
    }

    #[test]
    fn test_chained_lstrip_then_compiles() {
        let _rule = ChainedLstripThen::new();
    }
    */

    #[test]
    fn test_edge_case_any_alone_compiles() {
        let _rule = EdgeCaseAnyAlone::new();
    }

    #[test]
    fn test_edge_case_never_alone_compiles() {
        let _rule = EdgeCaseNeverAlone::new();
    }

    #[test]
    fn test_edge_case_not_any_compiles() {
        let _rule = EdgeCaseNotAny::new();
    }

    #[test]
    fn test_edge_case_any_then_never_compiles() {
        let _rule = EdgeCaseAnyThenNever::new();
    }

    #[test]
    fn test_edge_case_parenthesized_any_compiles() {
        let _rule = EdgeCaseParenthesizedAny::new();
    }

    #[test]
    fn test_edge_case_multiple_parens_compiles() {
        let _rule = EdgeCaseMultipleParens::new();
    }
}
