# Tokenization

## Algorithm

 1. If at end of file, return EOF.

 2. If at whitespace, consume it, then continue from 1.

 3. If at a special character, consume it and return SPECIAL.

 4. If at a digit character, consume it. Then continue as below,
    returning the appropriate type of number token.

    NOTE: These rules will produce malformed numbers if such exist
          in the source, such as 123foo.

    4.1. If at x or X, consume word characters, return HEX_INTEGER.

    4.2. If at b or B, consume word characters, return BIN_INTEGER.

    4.3. Consume decimal digits and underscore.

    4.4. If at dot: if followed by whitespace return DEC_INTEGER,
         otherwise consume the dot and the digits plus possible
         underscores.

    4.5. If at e or f, consume. If at + or -, consume. Consume word
         characters. For e return DOUBLE_FLOAT, for f return
         SINGLE_FLOAT.

    4.6. Consume word characters. If consumed a dot earlier, return
         DOUBLE_FLOAT, otherwise DEC_INTEGER.

 5. If at --- consume until --- and return BLOCK_COMMENT.

 6. If at -- consume to end of line and return COMMENT.

 7. If at """ consume until non-escaped """ and return BLOCK_STRING.

 8. If at " consume until non-escaped " and return STRING.

 9. If at a word character, until eof or non-word character.

    9.1 If the word is immediately followed by a single colon (ie. not double)
        and whitespace, consume the colon and return KEYWORD.

    9.2 If the word is immediately followed by a dot followed by a word character
        consume the dot and continue parsing the word at step 9.

    9.3 Otherwise return WORD.

10. At a sigil character, consume sigil characters and return SIGIL.

## Number characters

- Alphanumeric characters
- Underscore

## Special Characters

    (){}[],;$#

## Terminating characters

- Whitespace
- Special characters

## Word characters

- Alphanumeric characters (including subscripts and superscripts)
- Underscore

## Sigil characters

Non-word characters that are not terminating characters.

## Notes

- Since the tokenization rules don't contain any error conditions it
  will be possible to write user-level parsers that transition to an
  entirely different language.

- Similarly, while I hope this tokenization is simple enough to
  survive, since tokenization and parsing is handled step-by-step a
  syntax marker like

        syntax: foolang_v1

  can cause the parser to change the scanner into different mode.

- Annotations:

        foo::File <finalize>

  => Type(InstanceVariable("foo"), Global("File")) | SIGIL(<) WORD(finalize) SIGIL(>)

  To have SIGIL(<) work as postfix, it need cannot be applied to operators.
  That's OK as far as limitations go, I guess.

  Then SIGIL(<) parser will see that it is unbalanced to right, which means it is an
  annotation and not a less-than operator.

  Then it consumes until SIGIL(>) and checks that that was unbalanced to left.

