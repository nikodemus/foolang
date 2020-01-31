;;; c:/Users/nikod/src/foolang/foolang.el -*- lexical-binding: t; -*-

(require 'cl)

(defconst foolang-syntax-table (make-syntax-table))

(defun foolang-init-syntax-table (table)
  (modify-syntax-entry ?_ "w" table)
  (modify-syntax-entry ?\" "\"" table)
  (modify-syntax-entry ?\; "." table)
  (modify-syntax-entry ?. "." table)
  (modify-syntax-entry ?-  ". 12" table)
  (modify-syntax-entry ?-  "_ 12" table)
  (modify-syntax-entry ?\n ">" table))

;; Done this way so it can be mutated on the fly for development
(foolang-init-syntax-table foolang-syntax-table)

(define-derived-mode foolang-mode prog-mode "Foolang Mode"
  :syntax-table foolang-syntax-table
  (font-lock-fontify-buffer)
  (make-local-variable 'foolang-indent-offset)
  (set (make-local-variable 'indent-line-function) 'foolang-indent-line))

(add-to-list 'auto-mode-alist '("\\.foo" . foolang-mode))

(defvar foolang-indent-offset 4)

(setq foolang--indent-rules nil)

(cl-defmacro def-foolang-indent (name (col base stack ctx) &body body)
  `(let ((rule (list
                (lambda (,ctx) ,@(cdr (assoc :after body)))
                (lambda (,col ,base ,stack ,ctx) ,@(cdr (assoc :indent body)))))
        (old (assoc-string ,name foolang--indent-rules)))
     (if old
         (setcdr old rule)
       (push (cons ,name rule) foolang--indent-rules))))

(def-foolang-indent "-- comment" (col base stack ctx)
  (:after
    (looking-at "\\s-*--"))
  (:indent
   (list col base stack ctx)))

(def-foolang-indent "class" (col base stack ctx)
  (:after
    (looking-at " *class\\s-*$"))
  (:indent
   (list (* 2 foolang-indent-offset)
         (* 2 foolang-indent-offset)
         stack
         :class)))

(def-foolang-indent "class Name {" (col base stack ctx)
  (:after
    (looking-at " *class\\s-+[A-Z]+\\s-*{\\s-*$"))
  (:indent
   (list (* 2 foolang-indent-offset)
         (* 2 foolang-indent-offset)
         (cons (cons foolang-indent-offset foolang-indent-offset)
               stack)
         :slots)))

(def-foolang-indent "class Name { slot" (col base stack ctx)
  (:after
    (looking-at " *class\\s-+[A-Z]+\\s-*{\\s-*\\w+$"))
  (:indent
   (search-forward "{")
   (let ((p (foolang--line-length-to-here)))
     (list (+ p 1) (+ p 1)
           (cons (cons foolang-indent-offset foolang-indent-offset)
                 stack)
           :slots))))

(def-foolang-indent "\\ end" (col base stack ctx)
  (:after
    (looking-at "[^\n]*\n\\s-*end\\>"))
  (:indent
   (list 0 0 nil :toplevel)))

(def-foolang-indent "expr exit list" (col base stack ctx)
  (:after
    (and (looking-at ".*\\w")
         (foolang--nesting-decreases-on-line)))
  (:indent
   (let ((top (car stack)))
     (list (car top) (cdr top) (cdr stack) ctx))))

(def-foolang-indent "expr \\ exit list" (col base stack ctx)
  (:after
    (save-excursion
      (when (foolang--looking-at-nonterminated-expr)
        (next-line)
        (beginning-of-line)
        (looking-at " *[]})]"))))
  (:indent
   (let ((top (car stack)))
     (list (car top) (cdr top) (cdr stack) ctx))))

(def-foolang-indent "enter list expr. or |...|" (col base stack ctx)
  (:after
    (and (foolang--nesting-increases-on-line)
         (or (foolang--looking-at-terminated-expr)
             (looking-at ".*|[^|]*|\\s-*$"))))
  (:indent
   (end-of-line)
   (backward-up-list)
   (let ((p (current-column)))
     (list (+ p 2) (+ p 2)
           (cons (cons col base) stack)
           ctx))))

(def-foolang-indent "enter list eol" (col base stack ctx)
  (:after
    (when (eq :body ctx)
      (looking-at ".*[[({]\\s-*$")))
  (:indent
   (list (+ col foolang-indent-offset)
         (+ col foolang-indent-offset)
         (cons (cons col base) stack)
         ctx)))

(def-foolang-indent "method name" (col base stack ctx)
  (:after
    (looking-at " *\\(class *\\)?method *\\w+\\s_* *$"))
  (:indent
   (list (+ col foolang-indent-offset)
         (+ col foolang-indent-offset)
         stack
         :body)))

(def-foolang-indent "method op arg" (col base stack ctx)
  (:after
    (looking-at " *\\(class *\\)?method *\\s_+ +\\w+ *$"))
  (:indent
   (list (+ col foolang-indent-offset)
         (+ col foolang-indent-offset)
         stack
         :body)))

(def-foolang-indent "method name: var \\ name:" (col base stack ctx)
  (:after
    (looking-at " *\\(class *\\)?method *\\(\\w+: *\\w+\\s-*\\)+ *\n\\s-*\\w+:"))
  (:indent
   (search-forward ":")
   (backward-word)
   (let ((p (current-column)))
     (list p
           (+ col foolang-indent-offset)
           stack
           :method))))

(def-foolang-indent "method name: var" (col base stack ctx)
  (:after
    (looking-at " *\\(class *\\)?method *\\(\\w+: *\\w+\\s-*\\)+ *$"))
  (:indent
   (list (+ col foolang-indent-offset)
         (+ col foolang-indent-offset)
         stack
         :body)))

(def-foolang-indent "tailing method var: name \\ expr" (col base stack ctx)
  (:after
    (when (eq :method ctx)
      (looking-at " *\\(\\w+: *\\w+\\s-*\\)+\n\\s-*\\w+")))
  (:indent
   (list base base stack :body)))

(def-foolang-indent "in-body expr name: expr \\ name:" (col base stack ctx)
  (:after
    (when (eq :body ctx)
      (and (foolang--looking-at-nonterminated-expr)
           (looking-at "[^\n]+:\\s-*\\w[^\n:]++\n\\s-*\\w+:"))))
  (:indent
   (search-forward ":")
   (backward-word)
   (let ((p (current-column)))
     (list p base stack ctx))))

(def-foolang-indent "in-body name: var \\ name: var" (col base stack ctx)
  (:after
    (when (eq :body ctx)
      (looking-at " *\\(\\w+: *\\w+\\s-*\\)+\n\\s-*\\w+:")))
  (:indent
   (list col base stack ctx)))

(def-foolang-indent "; expr \\ ;" (col base stack ctx)
  (:after
    (when (eq :body ctx)
      (and (foolang--looking-at-nonterminated-expr)
           (looking-at "\\s-*;[^\n]*\n\\s-*;"))))
  (:indent
   (list col base stack ctx)))

(def-foolang-indent "expr" (col base stack ctx)
  (:after
    (when (eq :body ctx)
      (foolang--looking-at-nonterminated-expr)))
  (:indent
   (list (+ col foolang-indent-offset) base stack ctx)))

(def-foolang-indent "{ expr." (col base stack ctx)
  (:after
    (when (eq :body ctx)
      (and (foolang--looking-at-terminated-expr)
           (foolang--nesting-increases-on-line))))
  (:indent
   (end-of-line)
   (backward-up-list)
   (let ((p (+ 2 (current-column))))
     (list p p (cons (cons col base) stack) ctx))))

(def-foolang-indent "expr." (col base stack ctx)
  (:after
    (when (eq :body ctx)
      (foolang--looking-at-terminated-expr)))
  (:indent
   (list base base stack ctx)))

;;;; Indentation engine

(setq foolang--debug-indentation nil)

(defun foolang--note (control &rest args)
  (when foolang--debug-indentation
    (with-current-buffer (get-buffer-create "*foolang-indentation*")
      (end-of-buffer)
      (insert (apply #'format control args))
      (insert "\n"))))

(defun foolang-indent-line ()
  (foolang--indent-line-number (line-number-at-pos) nil))

(defun foolang-indent-all ()
  (foolang--indent-line-number (line-number-at-pos) t))

(defun foolang--indent-line-number (line-number indent-all)
  (save-excursion
    (lexical-let ((base (foolang--find-indent-base)))
      (foolang--indent-to line-number base base nil nil indent-all))))

(defun foolang--find-indent-base ()
  "Search lines up until it find a 'base', meaning
   a class or method definition line, or top of buffer."
  (lexical-let ((base nil))
    (while (not (setq base (foolang--indent-base-or-nil)))
      (previous-line))
    (foolang--note "indent-base: %s on line %s" base (line-number-at-pos))
    base))

(defun foolang--indent-to (target col base stack ctx indent-all)
  (let ((now (line-number-at-pos)))
    ;; (foolang--note "line %s (target %s)" now target)
    (if (eql target now)
        (indent-line-to col)
      (when indent-all
        (indent-line-to col))
      (destructuring-bind (new-col new-base new-stack new-ctx)
          (foolang--compute-next-line-indent col base stack ctx)
        (next-line)
        (beginning-of-line)
        (foolang--indent-to target new-col new-base new-stack new-ctx indent-all)))))

(cl-defun foolang--compute-next-line-indent (col base stack ctx)
  (foolang--note "indent: '%s'" (foolang--current-line))
  (dolist (rule (reverse foolang--indent-rules))
    (if (funcall (second rule) ctx)
      (lexical-let ((indent (funcall (third rule) col base stack ctx)))
        (foolang--note "  => rule '%s' => %s" (first rule) indent)
        (return-from foolang--compute-next-line-indent indent))
      (foolang--note "  not '%s'" (first rule))))
  (foolang--note "  ! No rule to indent line after '%s'" (foolang--current-line))
  (list col base stack ctx))

(defun foolang--indent-base-or-nil ()
  (beginning-of-line)
  (cond ((foolang--looking-at-method) foolang-indent-offset)
        ((foolang--looking-at-class) 0)
        ((foolang--top-of-buffer) 0)))

(defun foolang--looking-at-method ()
  (looking-at " *\\(method\\|class *method\\)"))

(defun foolang--looking-at-class ()
  (looking-at " *class [A-Z]"))

(defun foolang--top-of-buffer ()
  (eql 1 (line-number-at-pos)))

(defun foolang--looking-at-nonterminated-expr ()
  (looking-at ".*[^\\.,]\\s-*\\(--.*\\)?$"))

(defun foolang--looking-at-terminated-expr ()
  (looking-at ".*[\\.,]\\s-*\\(--.*\\)?$"))

(defun foolang--nesting-decreases-on-line ()
  (save-excursion
    (beginning-of-line)
    (eql (line-number-at-pos)
         (condition-case nil
             (progn (backward-up-list)
                    (forward-list)
                    (line-number-at-pos))
           (error nil)))))

(defun foolang--nesting-increases-on-line ()
  (save-excursion
    (end-of-line)
    (eql (line-number-at-pos)
         (condition-case nil
             (progn (backward-up-list)
                    (line-number-at-pos))
           (error nil)))))

(defun foolang--line-length-to-here ()
  (save-excursion
    (- (current-column)
       (progn (back-to-indentation) (current-column)))))

(defun foolang--current-line ()
  (substring-no-properties (thing-at-point 'line)))

;;;; Indentation testing

(setq foolang--indentation-test-failures nil)

(with-current-buffer (get-buffer-create "*foolang-indentation*")
  (erase-buffer))

(cl-defmacro def-foolang-indent-test (name source target)
  (with-current-buffer (get-buffer-create "*foolang-indentation*")
    (end-of-buffer)
    (insert "\n--- " name " ---\n"))
  (lexical-let ((result (with-temp-buffer
                          (foolang-mode)
                          (setq indent-tabs-mode nil)
                          (insert source)
                          (foolang-indent-all)
                          (end-of-buffer)
                          (buffer-substring-no-properties 1 (point)))))
    (with-current-buffer (get-buffer "*foolang-indentation*")
      (end-of-buffer)
      (if (string= target result)
          (progn (insert "ok!")
                 (message "test %s ok" name))
        (setq foolang--indentation-test-failures t)
        (let ((p (point)))
          (insert "FAILED!\n")
          (insert "WANTED:\n")
          (insert target)
          (insert "\nGOT:\n")
          (insert result)
          (message "test %s FAILED:\n%s" name
                   (buffer-substring-no-properties p (point))))))))

(def-foolang-indent-test "class-indent-1"
  "
    class"
  "
class")

(def-foolang-indent-test "class-indent-2"
  "
    class Foo"
  "
class Foo")

(def-foolang-indent-test "class-indent-3"
  "
    class
   Foo"
  "
class
        Foo")

(def-foolang-indent-test "class-indent-4"
  "
class Foo { a }
method"
  "
class Foo { a }
    method")

(def-foolang-indent-test "class-indent-5"
  "
class Foo {
a
}"
  "
class Foo {
        a
    }")

(def-foolang-indent-test "class-indent-6"
  "
class Foo { a
b }"
  "
class Foo { a
            b }")

(def-foolang-indent-test "method-indent-1"
  "
class Foo { a }
method bar
42"
  "
class Foo { a }
    method bar
        42")

(def-foolang-indent-test "method-indent-2"
  "
class Foo { a }
method bar: x
x"
  "
class Foo { a }
    method bar: x
        x")

(def-foolang-indent-test "method-indent-3"
  "
class Foo { a }
method bar: x
quux: y
x + y"
  "
class Foo { a }
    method bar: x
           quux: y
        x + y")

(def-foolang-indent-test "method-indent-4"
  "
method bar: x quux: y
x + y"
  "
    method bar: x quux: y
        x + y")

(def-foolang-indent-test "method-indent-5"
  "
method prefix-
-(self value)"
  "
    method prefix-
        -(self value)")

(def-foolang-indent-test "body-indent-1"
  "
method bar
quux
zot"
  "
    method bar
        quux
            zot")

(def-foolang-indent-test "body-indent-2"
  "
method bar
quux
zot.
fii"
  "
    method bar
        quux
            zot.
        fii")

(def-foolang-indent-test "body-indent-3"
  "
method bar
zot { dint.
flint"
  "
    method bar
        zot { dint.
              flint")

(def-foolang-indent-test "body-indent-4"
  "
method bar
zotarionz bee: x foo: y neg
faa: z neg neg
aas: s"
  "
    method bar
        zotarionz bee: x foo: y neg
                  faa: z neg neg
                  aas: s")

(def-foolang-indent-test "body-indent-5"
  "
method bar
things
do: { |thing|
thing"
  "
    method bar
        things
            do: { |thing|
                  thing")

(def-foolang-indent-test "body-indent-6"
  "
method bar
things
; doStuff: x
; moreStuff: y"
  "
    method bar
        things
            ; doStuff: x
            ; moreStuff: y")

(def-foolang-indent-test "body-indent-7"
  "
method bar
P new: p x: {
let n = p ifTrue: { x } ifFalse: { y }.
Ouch of: n"
  "
    method bar
        P new: p x: {
            let n = p ifTrue: { x } ifFalse: { y }.
            Ouch of: n")

(def-foolang-indent-test "body-indent-8"
  "
method bar
-- XXX
let foo = Quux x: 1"
  "
    method bar
        -- XXX
        let foo = Quux x: 1")

(def-foolang-indent-test "body-indent-9"
  "
class method run: command in: system
-- XXX: decide full/short based on command-line
let benchmarks = Benchmarks output: system output
clock: system clock
full: False.
benchmarks run"
  "
    class method run: command in: system
        -- XXX: decide full/short based on command-line
        let benchmarks = Benchmarks output: system output
                                    clock: system clock
                                    full: False.
        benchmarks run")

(def-foolang-indent-test "body-indent-10"
  "
class method new: system
let compiler = Compiler new.
compiler define: \"system\" as: system.
self _input: system input
_output: system output
_compiler: compiler
_atEof: False
_value: False"
        "
    class method new: system
        let compiler = Compiler new.
        compiler define: \"system\" as: system.
        self _input: system input
             _output: system output
             _compiler: compiler
             _atEof: False
             _value: False")

(def-foolang-indent-test "body-indent-11"
  "
method read
let source = \"\".
{
let line = _input readline.
_atEof = line is False.
_atEof ifFalse: {
source = source append: line newline.
self _tryParse: source
}
} whileFalse"
  "
    method read
        let source = \"\".
        {
            let line = _input readline.
            _atEof = line is False.
            _atEof ifFalse: {
                source = source append: line newline.
                self _tryParse: source
            }
        } whileFalse")

(def-foolang-indent-test "body-indent-12"
  "
method readEvalPrint
{
-- Cascade just for fun.
self; prompt; read; eval; print
}
onError: { |error context|
_output println: \"ERROR: {error}\".
_output println: context }"
  "
    method readEvalPrint
        {
            -- Cascade just for fun.
            self; prompt; read; eval; print
        }
            onError: { |error context|
                       _output println: \"ERROR: {error}\".
                       _output println: context }")

(def-foolang-indent-test "body-indent-13"
  "
method testPrefix
assertForAll: (1 to: 10)
that: { |n|
let b = Box value: n.
-n == -b }
testing: \"custom prefix method\""
  "
    method testPrefix
        assertForAll: (1 to: 10)
        that: { |n|
                let b = Box value: n.
                -n == -b }
        testing: \"custom prefix method\"")

(def-foolang-indent-test "end-indent-1"
  "
method bar
42
end"
  "
    method bar
        42
end")

(with-current-buffer "*foolang-indentation*"
  (cond (foolang--indentation-test-failures
         (display-buffer "*foolang-indentation*")
         (user-error "Foolang indentation tests failed!"))
        (t
         (kill-buffer)
         (message "Foolang tests ok!"))))
