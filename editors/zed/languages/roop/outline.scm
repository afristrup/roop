(function_item
  (visibility)? @context
  "irrev"? @context
  "fn" @context
  name: (_) @name) @item

(extern_item
  (visibility)? @context
  "extern" @context
  "fn" @context
  name: (_) @name) @item

(test_item
  "test" @context
  name: (_) @name) @item

(bennett_item
  (visibility)? @context
  "bennett" @context
  "fn" @context
  name: (_) @name) @item

(struct_item
  (visibility)? @context
  "struct" @context
  name: (_) @name) @item

(enum_item
  (visibility)? @context
  "enum" @context
  name: (_) @name) @item

(session_item
  (visibility)? @context
  "session" @context
  name: (_) @name) @item

(mod_item
  "mod" @context
  name: (_) @name) @item
