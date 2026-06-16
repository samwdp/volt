[
  (arguments)
  (array)
  (class_body)
  (formal_parameters)
  (jsx_element)
  (jsx_expression)
  (jsx_self_closing_element)
  (object)
  (object_pattern)
  (parenthesized_expression)
  (statement_block)
] @indent.begin

(arrow_function
  body: (_) @_body
  (#not-kind-eq? @_body "statement_block")) @indent.begin

(assignment_expression
  right: (_) @_right
  (#not-kind-eq? @_right "arrow_function")) @indent.begin

(variable_declarator
  value: (_) @_value
  (#not-kind-eq? @_value "arrow_function" "call_expression")) @indent.begin

[
  ")"
  "}"
  "]"
  (jsx_closing_element)
  ">"
] @indent.branch

[
  "}"
  "]"
] @indent.end

(arguments
  ")" @indent.end)

(object
  "}" @indent.end)

(statement_block
  "}" @indent.end)

(jsx_closing_element
  ">" @indent.end)

(jsx_self_closing_element
  "/>" @indent.end)

[
  (comment)
  (ERROR)
] @indent.auto
