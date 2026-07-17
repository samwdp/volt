([
  (comment_block)
  (comment_line)
] @injection.content
  (#set! injection.language "comment"))

([
  (doc_block)
  (doc_line)
] @injection.content
  (#set! injection.language "doxygen"))

; TODO(ISS-022): string and print (numbered) format injection
