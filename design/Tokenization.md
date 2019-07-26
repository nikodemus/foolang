# Tokenization

## Algorithm

 1. If at end of file, return EOF.

 2. If at whitespace, consume it. If whitespace contained a newline,
    return NEWLINE, otherwise continue from 1.

 3. If at a special character, consume it and return SPECIAL.

 4. If at a numeric character, consume until terminating character and return NUMBER.

 5. If at --- consume until ---- and return BLOCK_COMMENT.

 6. If at -- consume to end of line and return COMMENT.

 7. If at """ consume until """ and return BLOCK_STRING.

 8. If at " consume until " and return STRING.

 9. If at a word character, consume the word. If the word is
    immediately followed by a single colon and whitespace, consume the
    colon and return KEYWORD, otherwise return WORD.

10. At a sigil character, consume the sigil and return SIGIL.

## Special Characters

    (){}[],;$#

## Terminating characters

- Whitespace
- Special characters

## Word characters

- Alphabetic characters (including subscripts and superscripts)
- Numeric characters
- Underscore

## Sigil characters

Non-alphabetic characters that are not special characters.

## Notes

- Since the tokenization rules don't contain any error conditions it
  will be possible to write user-level parsers that transition to an
  entirely different language.

- Similarly, while I hope this tokenization is simple enough to
  survive, since tokenization and parsing is handled step-by-step a
  syntax marker like

        syntax: foolang_v1

  can cause the parser to change the scanner into different mode.
