; Highlights for roop. Capture names are the ones Zed's themes know.

(comment) @comment

(integer) @number
(float) @number
(string_literal) @string
(byte_literal) @string
(boolean) @boolean
(empty) @constant

; Keywords
[
  "mod" "use" "as" "extern" "irrev" "enum" "struct" "fn" "build" "unbuild" "mut"
  "if" "else" "fi" "from" "loop" "until" "match" "assert" "expect"
  "ancilla" "borrow" "call" "uncall" "chan" "send" "recv" "push" "pop"
  "logged" "try" "catch_rollback"
  "test" "bennett"
  "session" "offer" "select" "checkpoint" "rec"
] @keyword
(visibility) @keyword
(end_behaviour) @keyword

; Types
(primitive_type) @type
(type_identifier) @type
"Stack" @type
(struct_item name: (identifier) @type)
(enum_item name: (identifier) @type)
(session_item name: (identifier) @type)
(generics (identifier) @type)
(array_type length: (identifier) @type)
(stack_type length: (identifier) @type)
(call_generics (identifier) @type)
(mod_item name: (identifier) @type)
(use_item (identifier) @type)
(use_name (identifier) @type)
(use_name alias: (identifier) @type)
(use_item alias: (identifier) @type)
(rec_behaviour name: (identifier) @variable)

; Enums
(variant) @variant
(variant_path enum: (identifier) @type variant: (identifier) @variant)

; Functions and tests
(function_item name: (identifier) @function)
(extern_item name: (identifier) @function)
(test_item name: (identifier) @function)
(bennett_item name: (identifier) @function target: (identifier) @function)
(call_statement function: (identifier) @function)

; Names
(parameter name: (identifier) @variable.special)
(ancilla_statement name: (identifier) @variable.special)
(ancilla_declaration name: (identifier) @variable.special)
(borrow_statement name: (identifier) @variable.special)
(chan_statement name: (identifier) @variable.special)
(field_declaration name: (identifier) @property)
(field_expression field: (identifier) @property)
(role name: (identifier) @property)
(branch label: (identifier) @property)

; Attributes such as #[parallel(cuda)]
(attribute) @attribute
(attribute name: (identifier) @attribute)
(attribute target: (identifier) @constant)

; Operators and punctuation
[
  "+" "-" "*" "/" "%" "==" "!=" "<" "<=" ">" ">=" "&&" "||" "!" "&"
  "+=" "-=" "^=" "*=" "/=" "%=" "=" "<=>" "<-" "->" "=>"
] @operator

["(" ")" "[" "]" "{" "}"] @punctuation.bracket
["," ";" ":" "::" "." "#"] @punctuation.delimiter
(wildcard) @constant
