Could I specify using a read-table approach?

1. If at end of file, return EOF.

2. If at whitespace, consume it. If whitespace contained a newline,
   return NEWLINE, otherwise continue from 1.

3. If at a special character, consume it and return SPECIAL.

5. If at a numeric character, consume the number and return NUMBER.

7. If at --- consume until ---- and return BLOCK_COMMENT.

6. If at -- consume to end of line and return LINE_COMMENT.

6. If at a sigil character, consume the sigil and return IDENTIFIER.

7. If at a word character, consume the word and return IDENTIFIER.

Special:

    (){}[],;$#.:

...then the _parser_ can handle all the tricky stuff.

Since we have exact source locations it is easy to make sure that whitespace
does not appear where not wanted, eg prohibiting foo : bar.
