// Kernel extraction of ts-rust diag.rs plus diagnostics/catalog.rs:
// only the messages the scanner files name, each with its catalog
// entry (code, category, key, text, flags) copied verbatim.
// Generated once by hand tools from the two files above; do not edit
// entry text without checking the catalog.
#![allow(non_upper_case_globals)]

use crate::diagnostics::{Category, Message};

pub static Unterminated_string_literal: &Message = &Message::catalog(
    1_002,
    Category::Error,
    "Unterminated_string_literal_1002",
    "Unterminated string literal.",
    false,
    false,
    false,
);

pub static X_0_expected: &Message = &Message::catalog(
    1_005,
    Category::Error,
    "_0_expected_1005",
    "'{0}' expected.",
    false,
    false,
    false,
);

pub static Asterisk_Slash_expected: &Message = &Message::catalog(
    1_010,
    Category::Error,
    "Asterisk_Slash_expected_1010",
    "'*/' expected.",
    false,
    false,
    false,
);

pub static Octal_literals_are_not_allowed_Use_the_syntax_0: &Message = &Message::catalog(
    1_121,
    Category::Error,
    "Octal_literals_are_not_allowed_Use_the_syntax_0_1121",
    "Octal literals are not allowed. Use the syntax '{0}'.",
    false,
    false,
    false,
);

pub static Digit_expected: &Message = &Message::catalog(
    1_124,
    Category::Error,
    "Digit_expected_1124",
    "Digit expected.",
    false,
    false,
    false,
);

pub static Hexadecimal_digit_expected: &Message = &Message::catalog(
    1_125,
    Category::Error,
    "Hexadecimal_digit_expected_1125",
    "Hexadecimal digit expected.",
    false,
    false,
    false,
);

pub static Unexpected_end_of_text: &Message = &Message::catalog(
    1_126,
    Category::Error,
    "Unexpected_end_of_text_1126",
    "Unexpected end of text.",
    false,
    false,
    false,
);

pub static Invalid_character: &Message = &Message::catalog(
    1_127,
    Category::Error,
    "Invalid_character_1127",
    "Invalid character.",
    false,
    false,
    false,
);

pub static Unterminated_template_literal: &Message = &Message::catalog(
    1_160,
    Category::Error,
    "Unterminated_template_literal_1160",
    "Unterminated template literal.",
    false,
    false,
    false,
);

pub static Unterminated_regular_expression_literal: &Message = &Message::catalog(
    1_161,
    Category::Error,
    "Unterminated_regular_expression_literal_1161",
    "Unterminated regular expression literal.",
    false,
    false,
    false,
);

pub static Binary_digit_expected: &Message = &Message::catalog(
    1_177,
    Category::Error,
    "Binary_digit_expected_1177",
    "Binary digit expected.",
    false,
    false,
    false,
);

pub static Octal_digit_expected: &Message = &Message::catalog(
    1_178,
    Category::Error,
    "Octal_digit_expected_1178",
    "Octal digit expected.",
    false,
    false,
    false,
);

pub static Merge_conflict_marker_encountered: &Message = &Message::catalog(
    1_185,
    Category::Error,
    "Merge_conflict_marker_encountered_1185",
    "Merge conflict marker encountered.",
    false,
    false,
    false,
);

pub static An_extended_Unicode_escape_value_must_be_between_0x0_and_0x10FFFF_inclusive: &Message = &Message::catalog(
    1_198,
    Category::Error,
    "An_extended_Unicode_escape_value_must_be_between_0x0_and_0x10FFFF_inclusive_1198",
    "An extended Unicode escape value must be between 0x0 and 0x10FFFF inclusive.",
    false,
    false,
    false,
);

pub static Unterminated_Unicode_escape_sequence: &Message = &Message::catalog(
    1_199,
    Category::Error,
    "Unterminated_Unicode_escape_sequence_1199",
    "Unterminated Unicode escape sequence.",
    false,
    false,
    false,
);

pub static An_identifier_or_keyword_cannot_immediately_follow_a_numeric_literal: &Message = &Message::catalog(
    1_351,
    Category::Error,
    "An_identifier_or_keyword_cannot_immediately_follow_a_numeric_literal_1351",
    "An identifier or keyword cannot immediately follow a numeric literal.",
    false,
    false,
    false,
);

pub static A_bigint_literal_cannot_use_exponential_notation: &Message = &Message::catalog(
    1_352,
    Category::Error,
    "A_bigint_literal_cannot_use_exponential_notation_1352",
    "A bigint literal cannot use exponential notation.",
    false,
    false,
    false,
);

pub static A_bigint_literal_must_be_an_integer: &Message = &Message::catalog(
    1_353,
    Category::Error,
    "A_bigint_literal_must_be_an_integer_1353",
    "A bigint literal must be an integer.",
    false,
    false,
    false,
);

pub static Did_you_mean_0: &Message = &Message::catalog(
    1_369,
    Category::Message,
    "Did_you_mean_0_1369",
    "Did you mean '{0}'?",
    false,
    false,
    false,
);

pub static Unexpected_token_Did_you_mean_or_rbrace: &Message = &Message::catalog(
    1_381,
    Category::Error,
    "Unexpected_token_Did_you_mean_or_rbrace_1381",
    "Unexpected token. Did you mean `{'}'}` or `&rbrace;`?",
    false,
    false,
    false,
);

pub static Unexpected_token_Did_you_mean_or_gt: &Message = &Message::catalog(
    1_382,
    Category::Error,
    "Unexpected_token_Did_you_mean_or_gt_1382",
    "Unexpected token. Did you mean `{'>'}` or `&gt;`?",
    false,
    false,
    false,
);

pub static Octal_escape_sequences_are_not_allowed_Use_the_syntax_0: &Message = &Message::catalog(
    1_487,
    Category::Error,
    "Octal_escape_sequences_are_not_allowed_Use_the_syntax_0_1487",
    "Octal escape sequences are not allowed. Use the syntax '{0}'.",
    false,
    false,
    false,
);

pub static Escape_sequence_0_is_not_allowed: &Message = &Message::catalog(
    1_488,
    Category::Error,
    "Escape_sequence_0_is_not_allowed_1488",
    "Escape sequence '{0}' is not allowed.",
    false,
    false,
    false,
);

pub static Decimals_with_leading_zeros_are_not_allowed: &Message = &Message::catalog(
    1_489,
    Category::Error,
    "Decimals_with_leading_zeros_are_not_allowed_1489",
    "Decimals with leading zeros are not allowed.",
    false,
    false,
    false,
);

pub static File_appears_to_be_binary: &Message = &Message::catalog(
    1_490,
    Category::Error,
    "File_appears_to_be_binary_1490",
    "File appears to be binary.",
    false,
    false,
    false,
);

pub static Unknown_regular_expression_flag: &Message = &Message::catalog(
    1_499,
    Category::Error,
    "Unknown_regular_expression_flag_1499",
    "Unknown regular expression flag.",
    false,
    false,
    false,
);

pub static Duplicate_regular_expression_flag: &Message = &Message::catalog(
    1_500,
    Category::Error,
    "Duplicate_regular_expression_flag_1500",
    "Duplicate regular expression flag.",
    false,
    false,
    false,
);

pub static This_regular_expression_flag_is_only_available_when_targeting_0_or_later: &Message = &Message::catalog(
    1_501,
    Category::Error,
    "This_regular_expression_flag_is_only_available_when_targeting_0_or_later_1501",
    "This regular expression flag is only available when targeting '{0}' or later.",
    false,
    false,
    false,
);

pub static The_Unicode_u_flag_and_the_Unicode_Sets_v_flag_cannot_be_set_simultaneously: &Message = &Message::catalog(
    1_502,
    Category::Error,
    "The_Unicode_u_flag_and_the_Unicode_Sets_v_flag_cannot_be_set_simultaneously_1502",
    "The Unicode (u) flag and the Unicode Sets (v) flag cannot be set simultaneously.",
    false,
    false,
    false,
);

pub static Named_capturing_groups_are_only_available_when_targeting_ES2018_or_later: &Message = &Message::catalog(
    1_503,
    Category::Error,
    "Named_capturing_groups_are_only_available_when_targeting_ES2018_or_later_1503",
    "Named capturing groups are only available when targeting 'ES2018' or later.",
    false,
    false,
    false,
);

pub static Subpattern_flags_must_be_present_when_there_is_a_minus_sign: &Message = &Message::catalog(
    1_504,
    Category::Error,
    "Subpattern_flags_must_be_present_when_there_is_a_minus_sign_1504",
    "Subpattern flags must be present when there is a minus sign.",
    false,
    false,
    false,
);

pub static Incomplete_quantifier_Digit_expected: &Message = &Message::catalog(
    1_505,
    Category::Error,
    "Incomplete_quantifier_Digit_expected_1505",
    "Incomplete quantifier. Digit expected.",
    false,
    false,
    false,
);

pub static Numbers_out_of_order_in_quantifier: &Message = &Message::catalog(
    1_506,
    Category::Error,
    "Numbers_out_of_order_in_quantifier_1506",
    "Numbers out of order in quantifier.",
    false,
    false,
    false,
);

pub static There_is_nothing_available_for_repetition: &Message = &Message::catalog(
    1_507,
    Category::Error,
    "There_is_nothing_available_for_repetition_1507",
    "There is nothing available for repetition.",
    false,
    false,
    false,
);

pub static Unexpected_0_Did_you_mean_to_escape_it_with_backslash: &Message = &Message::catalog(
    1_508,
    Category::Error,
    "Unexpected_0_Did_you_mean_to_escape_it_with_backslash_1508",
    "Unexpected '{0}'. Did you mean to escape it with backslash?",
    false,
    false,
    false,
);

pub static This_regular_expression_flag_cannot_be_toggled_within_a_subpattern: &Message = &Message::catalog(
    1_509,
    Category::Error,
    "This_regular_expression_flag_cannot_be_toggled_within_a_subpattern_1509",
    "This regular expression flag cannot be toggled within a subpattern.",
    false,
    false,
    false,
);

pub static X_k_must_be_followed_by_a_capturing_group_name_enclosed_in_angle_brackets: &Message = &Message::catalog(
    1_510,
    Category::Error,
    "k_must_be_followed_by_a_capturing_group_name_enclosed_in_angle_brackets_1510",
    "'\\k' must be followed by a capturing group name enclosed in angle brackets.",
    false,
    false,
    false,
);

pub static X_q_is_only_available_inside_character_class: &Message = &Message::catalog(
    1_511,
    Category::Error,
    "q_is_only_available_inside_character_class_1511",
    "'\\q' is only available inside character class.",
    false,
    false,
    false,
);

pub static X_c_must_be_followed_by_an_ASCII_letter: &Message = &Message::catalog(
    1_512,
    Category::Error,
    "c_must_be_followed_by_an_ASCII_letter_1512",
    "'\\c' must be followed by an ASCII letter.",
    false,
    false,
    false,
);

pub static Undetermined_character_escape: &Message = &Message::catalog(
    1_513,
    Category::Error,
    "Undetermined_character_escape_1513",
    "Undetermined character escape.",
    false,
    false,
    false,
);

pub static Expected_a_capturing_group_name: &Message = &Message::catalog(
    1_514,
    Category::Error,
    "Expected_a_capturing_group_name_1514",
    "Expected a capturing group name.",
    false,
    false,
    false,
);

pub static Named_capturing_groups_with_the_same_name_must_be_mutually_exclusive_to_each_other: &Message = &Message::catalog(
    1_515,
    Category::Error,
    "Named_capturing_groups_with_the_same_name_must_be_mutually_exclusive_to_each_other_1515",
    "Named capturing groups with the same name must be mutually exclusive to each other.",
    false,
    false,
    false,
);

pub static A_character_class_range_must_not_be_bounded_by_another_character_class: &Message = &Message::catalog(
    1_516,
    Category::Error,
    "A_character_class_range_must_not_be_bounded_by_another_character_class_1516",
    "A character class range must not be bounded by another character class.",
    false,
    false,
    false,
);

pub static Range_out_of_order_in_character_class: &Message = &Message::catalog(
    1_517,
    Category::Error,
    "Range_out_of_order_in_character_class_1517",
    "Range out of order in character class.",
    false,
    false,
    false,
);

pub static Anything_that_would_possibly_match_more_than_a_single_character_is_invalid_inside_a_negated_character_class: &Message = &Message::catalog(
    1_518,
    Category::Error,
    "Anything_that_would_possibly_match_more_than_a_single_character_is_invalid_inside_a_negated_characte_1518",
    "Anything that would possibly match more than a single character is invalid inside a negated character class.",
    false,
    false,
    false,
);

pub static Operators_must_not_be_mixed_within_a_character_class_Wrap_it_in_a_nested_class_instead: &Message = &Message::catalog(
    1_519,
    Category::Error,
    "Operators_must_not_be_mixed_within_a_character_class_Wrap_it_in_a_nested_class_instead_1519",
    "Operators must not be mixed within a character class. Wrap it in a nested class instead.",
    false,
    false,
    false,
);

pub static Expected_a_class_set_operand: &Message = &Message::catalog(
    1_520,
    Category::Error,
    "Expected_a_class_set_operand_1520",
    "Expected a class set operand.",
    false,
    false,
    false,
);

pub static X_q_must_be_followed_by_string_alternatives_enclosed_in_braces: &Message = &Message::catalog(
    1_521,
    Category::Error,
    "q_must_be_followed_by_string_alternatives_enclosed_in_braces_1521",
    "'\\q' must be followed by string alternatives enclosed in braces.",
    false,
    false,
    false,
);

pub static A_character_class_must_not_contain_a_reserved_double_punctuator_Did_you_mean_to_escape_it_with_backslash: &Message = &Message::catalog(
    1_522,
    Category::Error,
    "A_character_class_must_not_contain_a_reserved_double_punctuator_Did_you_mean_to_escape_it_with_backs_1522",
    "A character class must not contain a reserved double punctuator. Did you mean to escape it with backslash?",
    false,
    false,
    false,
);

pub static Expected_a_Unicode_property_name: &Message = &Message::catalog(
    1_523,
    Category::Error,
    "Expected_a_Unicode_property_name_1523",
    "Expected a Unicode property name.",
    false,
    false,
    false,
);

pub static Unknown_Unicode_property_name: &Message = &Message::catalog(
    1_524,
    Category::Error,
    "Unknown_Unicode_property_name_1524",
    "Unknown Unicode property name.",
    false,
    false,
    false,
);

pub static Expected_a_Unicode_property_value: &Message = &Message::catalog(
    1_525,
    Category::Error,
    "Expected_a_Unicode_property_value_1525",
    "Expected a Unicode property value.",
    false,
    false,
    false,
);

pub static Unknown_Unicode_property_value: &Message = &Message::catalog(
    1_526,
    Category::Error,
    "Unknown_Unicode_property_value_1526",
    "Unknown Unicode property value.",
    false,
    false,
    false,
);

pub static Expected_a_Unicode_property_name_or_value: &Message = &Message::catalog(
    1_527,
    Category::Error,
    "Expected_a_Unicode_property_name_or_value_1527",
    "Expected a Unicode property name or value.",
    false,
    false,
    false,
);

pub static Any_Unicode_property_that_would_possibly_match_more_than_a_single_character_is_only_available_when_the_Unicode_Sets_v_flag_is_set: &Message = &Message::catalog(
    1_528,
    Category::Error,
    "Any_Unicode_property_that_would_possibly_match_more_than_a_single_character_is_only_available_when_t_1528",
    "Any Unicode property that would possibly match more than a single character is only available when the Unicode Sets (v) flag is set.",
    false,
    false,
    false,
);

pub static Unknown_Unicode_property_name_or_value: &Message = &Message::catalog(
    1_529,
    Category::Error,
    "Unknown_Unicode_property_name_or_value_1529",
    "Unknown Unicode property name or value.",
    false,
    false,
    false,
);

pub static Unicode_property_value_expressions_are_only_available_when_the_Unicode_u_flag_or_the_Unicode_Sets_v_flag_is_set: &Message = &Message::catalog(
    1_530,
    Category::Error,
    "Unicode_property_value_expressions_are_only_available_when_the_Unicode_u_flag_or_the_Unicode_Sets_v__1530",
    "Unicode property value expressions are only available when the Unicode (u) flag or the Unicode Sets (v) flag is set.",
    false,
    false,
    false,
);

pub static X_0_must_be_followed_by_a_Unicode_property_value_expression_enclosed_in_braces: &Message = &Message::catalog(
    1_531,
    Category::Error,
    "_0_must_be_followed_by_a_Unicode_property_value_expression_enclosed_in_braces_1531",
    "'\\{0}' must be followed by a Unicode property value expression enclosed in braces.",
    false,
    false,
    false,
);

pub static There_is_no_capturing_group_named_0_in_this_regular_expression: &Message = &Message::catalog(
    1_532,
    Category::Error,
    "There_is_no_capturing_group_named_0_in_this_regular_expression_1532",
    "There is no capturing group named '{0}' in this regular expression.",
    false,
    false,
    false,
);

pub static This_backreference_refers_to_a_group_that_does_not_exist_There_are_only_0_capturing_groups_in_this_regular_expression: &Message = &Message::catalog(
    1_533,
    Category::Error,
    "This_backreference_refers_to_a_group_that_does_not_exist_There_are_only_0_capturing_groups_in_this_r_1533",
    "This backreference refers to a group that does not exist. There are only {0} capturing groups in this regular expression.",
    false,
    false,
    false,
);

pub static This_backreference_refers_to_a_group_that_does_not_exist_There_are_no_capturing_groups_in_this_regular_expression: &Message = &Message::catalog(
    1_534,
    Category::Error,
    "This_backreference_refers_to_a_group_that_does_not_exist_There_are_no_capturing_groups_in_this_regul_1534",
    "This backreference refers to a group that does not exist. There are no capturing groups in this regular expression.",
    false,
    false,
    false,
);

pub static This_character_cannot_be_escaped_in_a_regular_expression: &Message = &Message::catalog(
    1_535,
    Category::Error,
    "This_character_cannot_be_escaped_in_a_regular_expression_1535",
    "This character cannot be escaped in a regular expression.",
    false,
    false,
    false,
);

pub static Octal_escape_sequences_and_backreferences_are_not_allowed_in_a_character_class_If_this_was_intended_as_an_escape_sequence_use_the_syntax_0_instead: &Message = &Message::catalog(
    1_536,
    Category::Error,
    "Octal_escape_sequences_and_backreferences_are_not_allowed_in_a_character_class_If_this_was_intended__1536",
    "Octal escape sequences and backreferences are not allowed in a character class. If this was intended as an escape sequence, use the syntax '{0}' instead.",
    false,
    false,
    false,
);

pub static Decimal_escape_sequences_and_backreferences_are_not_allowed_in_a_character_class: &Message = &Message::catalog(
    1_537,
    Category::Error,
    "Decimal_escape_sequences_and_backreferences_are_not_allowed_in_a_character_class_1537",
    "Decimal escape sequences and backreferences are not allowed in a character class.",
    false,
    false,
    false,
);

pub static Unicode_escape_sequences_are_only_available_when_the_Unicode_u_flag_or_the_Unicode_Sets_v_flag_is_set: &Message = &Message::catalog(
    1_538,
    Category::Error,
    "Unicode_escape_sequences_are_only_available_when_the_Unicode_u_flag_or_the_Unicode_Sets_v_flag_is_se_1538",
    "Unicode escape sequences are only available when the Unicode (u) flag or the Unicode Sets (v) flag is set.",
    false,
    false,
    false,
);

pub static Numeric_separators_are_not_allowed_here: &Message = &Message::catalog(
    6_188,
    Category::Error,
    "Numeric_separators_are_not_allowed_here_6188",
    "Numeric separators are not allowed here.",
    false,
    false,
    false,
);

pub static Multiple_consecutive_numeric_separators_are_not_permitted: &Message = &Message::catalog(
    6_189,
    Category::Error,
    "Multiple_consecutive_numeric_separators_are_not_permitted_6189",
    "Multiple consecutive numeric separators are not permitted.",
    false,
    false,
    false,
);

pub static X_can_only_be_used_at_the_start_of_a_file: &Message = &Message::catalog(
    18_026,
    Category::Error,
    "can_only_be_used_at_the_start_of_a_file_18026",
    "'#!' can only be used at the start of a file.",
    false,
    false,
    false,
);

pub static Regular_expression_pattern_modifiers_are_only_available_when_targeting_0_or_later: &Message = &Message::catalog(
    18_062,
    Category::Error,
    "Regular_expression_pattern_modifiers_are_only_available_when_targeting_0_or_later_18062",
    "Regular expression pattern modifiers are only available when targeting '{0}' or later.",
    false,
    false,
    false,
);

pub static Duplicate_named_capturing_groups_are_only_available_when_targeting_0_or_later: &Message = &Message::catalog(
    18_063,
    Category::Error,
    "Duplicate_named_capturing_groups_are_only_available_when_targeting_0_or_later_18063",
    "Duplicate named capturing groups are only available when targeting '{0}' or later.",
    false,
    false,
    false,
);
