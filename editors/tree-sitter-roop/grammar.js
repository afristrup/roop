// Tree-sitter grammar for roop. It follows crates/roop-syntax/src/parser, and
// is meant for highlighting, outlines and structural editing, so it accepts a
// little more than the compiler does. Words the compiler treats as contextual
// (test, expect, bennett, session, offer, select, checkpoint, rec, end, Stack)
// are keywords here, so they cannot be used as names in an editor buffer.

const PREC = {
  or: 1,
  and: 2,
  equal: 3,
  compare: 4,
  add: 5,
  multiply: 6,
  cast: 7,
  unary: 8,
};

function commaSep(rule) {
  return optional(commaSep1(rule));
}

function commaSep1(rule) {
  return seq(rule, repeat(seq(",", rule)), optional(","));
}

module.exports = grammar({
  name: "roop",

  extras: ($) => [/\s/, $.comment],

  word: ($) => $.identifier,

  rules: {
    source_file: ($) => repeat($._item),

    comment: (_) => token(seq("//", /[^\n]*/)),

    identifier: (_) => /[A-Za-z][A-Za-z0-9_]*|_[A-Za-z0-9_]+/,
    integer: (_) => /[0-9]+/,
    float: (_) => /[0-9]+\.[0-9]+/,
    boolean: (_) => choice("true", "false"),
    string_literal: (_) => token(seq('"', repeat(choice(/[^"\\\n]/, /\\./)), '"')),
    byte_literal: (_) => token(seq("b'", choice(/[^'\\\n]/, /\\./), "'")),

    // Items

    _item: ($) =>
      choice(
        $.mod_item,
        $.use_item,
        $.enum_item,
        $.struct_item,
        $.function_item,
        $.extern_item,
        $.test_item,
        $.bennett_item,
        $.einsum_item,
        $.session_item,
      ),

    visibility: (_) => "pub",

    mod_item: ($) => seq("mod", field("name", $.identifier), ";"),

    use_item: ($) =>
      seq(
        optional($.visibility),
        "use",
        $.identifier,
        repeat(seq("::", choice($.identifier, "*", $.use_group))),
        optional(seq("as", field("alias", $.identifier))),
        ";",
      ),

    use_group: ($) => seq("{", commaSep($.use_name), "}"),

    use_name: ($) =>
      seq($.identifier, optional(seq("as", field("alias", $.identifier)))),

    enum_item: ($) =>
      seq(
        optional($.visibility),
        "enum",
        field("name", $.identifier),
        "{",
        commaSep($.variant),
        "}",
      ),

    variant: ($) => $.identifier,

    struct_item: ($) =>
      seq(
        optional($.visibility),
        "struct",
        field("name", $.identifier),
        "{",
        optional(commaSep1($.field_declaration)),
        repeat($.build_function),
        "}",
      ),

    field_declaration: ($) =>
      seq(field("name", $.identifier), ":", field("type", $._type)),

    build_function: ($) =>
      seq(choice("build", "unbuild"), $.parameters, field("body", $.block)),

    function_item: ($) =>
      seq(
        optional($.visibility),
        optional("irrev"),
        "fn",
        field("name", $.identifier),
        optional($.generics),
        $.parameters,
        field("body", $.block),
      ),

    extern_item: ($) =>
      seq(
        optional($.visibility),
        "extern",
        optional("world"),
        "fn",
        field("name", $.identifier),
        optional($.generics),
        $.parameters,
        ";",
      ),

    generics: ($) => seq("<", commaSep($.identifier), ">"),

    parameters: ($) => seq("(", commaSep($.parameter), ")"),

    parameter: ($) =>
      seq(field("name", $.identifier), ":", field("type", $._type)),

    test_item: ($) =>
      seq(
        "test",
        field("name", $.identifier),
        "{",
        optional(seq(commaSep1($.parameter), ";")),
        repeat($._statement),
        "}",
      ),

    bennett_item: ($) =>
      seq(
        optional($.visibility),
        "bennett",
        "fn",
        field("name", $.identifier),
        "=",
        field("target", $.identifier),
        ";",
      ),

    einsum_item: ($) =>
      seq(
        optional($.visibility),
        "einsum",
        "fn",
        field("name", $.identifier),
        ":",
        field("type", $._type),
        "=",
        field("subscripts", $.string_literal),
        ";",
      ),

    session_item: ($) =>
      seq(
        optional($.visibility),
        "session",
        field("name", $.identifier),
        "{",
        repeat1($.role),
        "}",
      ),

    role: ($) =>
      seq(field("name", $.identifier), ":", $._behaviour, ";"),

    _behaviour: ($) =>
      choice($.end_behaviour, $.choice_behaviour, $.rec_behaviour, $.identifier),

    end_behaviour: (_) => "end",

    choice_behaviour: ($) =>
      seq(
        optional("checkpoint"),
        choice("offer", "select"),
        "{",
        commaSep($.branch),
        "}",
      ),

    rec_behaviour: ($) =>
      seq("rec", field("name", $.identifier), "{", $._behaviour, "}"),

    branch: ($) => seq(field("label", $.identifier), ":", $._behaviour),

    // Types

    _type: ($) =>
      choice(
        $.primitive_type,
        $.type_identifier,
        $.reference_type,
        $.array_type,
        $.stack_type,
      ),

    primitive_type: (_) => choice("i64", "f64", "u8", "bool"),

    type_identifier: ($) => $.identifier,

    reference_type: ($) =>
      seq("&", optional("mut"), field("type", $._type)),

    array_type: ($) =>
      seq("[", field("element", $._type), ";", field("length", $._length), "]"),

    stack_type: ($) =>
      seq(
        "Stack",
        "<",
        field("element", $._type),
        ",",
        field("length", $._length),
        ">",
      ),

    _length: ($) => choice($.integer, $.identifier),

    // Statements

    block: ($) => seq("{", repeat($._statement), "}"),

    attribute: ($) =>
      seq(
        "#",
        "[",
        field("name", $.identifier),
        optional(seq("(", field("target", $.identifier), ")")),
        "]",
      ),

    _statement: ($) =>
      seq(
        repeat($.attribute),
        choice(
          $.update_statement,
          $.swap_statement,
          $.overwrite_statement,
          $.expect_statement,
          $.if_statement,
          $.from_statement,
          $.match_statement,
          $.ancilla_statement,
          $.ancilla_declaration,
          $.borrow_statement,
          $.call_statement,
          $.chan_statement,
          $.send_statement,
          $.recv_statement,
          $.push_statement,
          $.pop_statement,
          $.keep_statement,
          $.logged_statement,
          $.irrev_statement,
          $.try_statement,
          $.block,
        ),
      ),

    update_statement: ($) =>
      seq(
        field("target", $._place),
        field("operator", choice("+=", "-=", "^=", "*=", "/=")),
        field("value", $._expression),
        ";",
      ),

    swap_statement: ($) =>
      seq($._place, "<=>", $._place, ";"),

    overwrite_statement: ($) =>
      seq(
        field("target", $._place),
        field("operator", choice("=", "%=")),
        field("value", $._expression),
        ";",
      ),

    expect_statement: ($) => seq("expect", $._expression, ";"),

    if_statement: ($) =>
      seq(
        "if",
        field("condition", $._expression),
        field("then", $.block),
        optional(seq("else", field("else", $.block))),
        "fi",
        field("exit", $._expression),
        ";",
      ),

    from_statement: ($) =>
      seq(
        "from",
        field("entry", $._expression),
        field("body", $.block),
        optional(seq("loop", field("step", $.block))),
        "until",
        field("exit", $._expression),
        ";",
      ),

    match_statement: ($) =>
      seq("match", field("value", $._expression), "{", repeat1($.match_arm), "}"),

    match_arm: ($) =>
      seq(
        field("pattern", $._pattern),
        "=>",
        field("body", $.block),
        "assert",
        field("exit", $._expression),
        ";",
      ),

    _pattern: ($) =>
      choice($.integer, $.boolean, $.variant_path, alias("_", $.wildcard)),

    ancilla_statement: ($) =>
      seq(
        "ancilla",
        field("name", $.identifier),
        ":",
        field("type", $._type),
        "=",
        field("init", $._expression),
        field("body", $.block),
      ),

    ancilla_declaration: ($) =>
      seq(
        "ancilla",
        field("name", $.identifier),
        ":",
        field("type", $._type),
        "=",
        field("init", $._expression),
        ";",
      ),

    borrow_statement: ($) =>
      seq(
        "borrow",
        field("name", $.identifier),
        "=",
        field("source", $._place),
        field("body", $.block),
      ),

    call_statement: ($) =>
      seq(
        choice("call", "uncall"),
        field("function", $.identifier),
        optional($.call_generics),
        $.arguments,
        ";",
      ),

    call_generics: ($) => seq("<", commaSep($._length), ">"),

    arguments: ($) => seq("(", commaSep($._expression), ")"),

    chan_statement: ($) =>
      seq(
        "chan",
        field("name", $.identifier),
        ":",
        field("type", $._type),
        field("body", $.block),
      ),

    send_statement: ($) =>
      seq("send", field("channel", $.identifier), "<-", $._place, ";"),

    recv_statement: ($) =>
      seq("recv", field("channel", $.identifier), "->", $._place, ";"),

    push_statement: ($) => seq("push", $._place, "<-", $._place, ";"),

    pop_statement: ($) => seq("pop", $._place, "->", $._place, ";"),

    keep_statement: ($) => seq("keep", $._place, ";"),

    logged_statement: ($) =>
      seq("logged", field("history", $._place), field("body", $.block)),

    irrev_statement: ($) => seq("irrev", field("body", $.block)),

    try_statement: ($) =>
      seq(
        "try",
        field("body", $.block),
        "catch_rollback",
        field("handler", $.block),
        optional(seq("->", field("outcome", $._place), ";")),
      ),

    // Expressions

    _place: ($) =>
      choice($.identifier, $.field_expression, $.index_expression),

    field_expression: ($) =>
      prec.left(
        seq(field("value", $._place), ".", field("field", $.identifier)),
      ),

    index_expression: ($) =>
      prec.left(seq(field("value", $._place), "[", $._expression, "]")),

    _expression: ($) =>
      choice(
        $.integer,
        $.float,
        $.string_literal,
        $.byte_literal,
        $.boolean,
        $.empty,
        $.variant_path,
        $._place,
        $.unary_expression,
        $.cast_expression,
        $.binary_expression,
        $.parenthesized_expression,
      ),

    empty: (_) => "empty",

    variant_path: ($) =>
      seq(
        field("enum", $.identifier),
        "::",
        field("variant", $.identifier),
      ),

    parenthesized_expression: ($) => seq("(", $._expression, ")"),

    cast_expression: ($) =>
      prec.left(PREC.cast, seq($._expression, "as", field("type", $._type))),

    unary_expression: ($) =>
      prec(PREC.unary, seq(choice("-", "!"), $._expression)),

    binary_expression: ($) =>
      choice(
        ...[
          ["||", PREC.or],
          ["&&", PREC.and],
          ["==", PREC.equal],
          ["!=", PREC.equal],
          ["<", PREC.compare],
          ["<=", PREC.compare],
          [">", PREC.compare],
          [">=", PREC.compare],
          ["+", PREC.add],
          ["-", PREC.add],
          ["*", PREC.multiply],
          ["/", PREC.multiply],
          ["%", PREC.multiply],
        ].map(([operator, precedence]) =>
          prec.left(
            precedence,
            seq(
              field("left", $._expression),
              field("operator", operator),
              field("right", $._expression),
            ),
          ),
        ),
      ),
  },
});
