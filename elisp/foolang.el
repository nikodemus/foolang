;;; c:/Users/nikod/src/foolang/foolang.el -*- lexical-binding: t; -*-

(require 'cl)

(defvar foolang-syntax-table)
(setq foolang-syntax-table (make-syntax-table))

(defun foolang-init-syntax-table (table)
  (modify-syntax-entry ?_ "w" table)
  (modify-syntax-entry ?\" "\"" table)
  (modify-syntax-entry ?\; "." table)
  (modify-syntax-entry ?. "." table)
  (modify-syntax-entry ?-  ". 12" table)
  ;; _ is symbol constitutient, 12 is first and second character of
  ;; two-character comment start (comment type a)
  (modify-syntax-entry ?-  "_ 12" table)
  ;; > is comment end
  (modify-syntax-entry ?\n ">" table))

;; Done this way so it can be mutated on the fly for development
(foolang-init-syntax-table foolang-syntax-table)

(defconst foolang--syntax-propertize-rules
  (syntax-propertize-precompile-rules
   ("---" (0 "! b"))))

(defun foolang--syntax-propertize (start end)
  (goto-char start)
  (funcall (syntax-propertize-rules foolang--syntax-propertize-rules)
           start end))

(defun foolang--syntax-propertize-extend-block-comments (start end)
  (save-excursion
    (save-restriction
      (widen)
      (narrow-to-region (point-min) start)
      (goto-char (point-min))
      ;; Count the number of --- comment fences before the start: if odd, move
      ;; region start to the beginning of the line with the first one.
      (let ((n 0))
        (while (looking-at "\\(.\\|\n\\)*?---")
          (goto-char (match-end 0))
          (incf n))
        (if (eql 0 (% n 2))
            ;; Even number of fences, everything is fine.
            (cons start end)
          ;; Odd number of fences, widen
          (cons (match-beginning 0) end))))))

(define-derived-mode foolang-mode prog-mode "Foolang Mode"
  :syntax-table foolang-syntax-table
  (make-local-variable 'foolang-indent-offset)
  (setq-local indent-line-function 'foolang-indent-line)
  (setq-local syntax-propertize-function 'foolang--syntax-propertize)
  (setq-local syntax-propertize-extend-region-functions
              '(syntax-propertize-wholelines
                foolang--syntax-propertize-extend-block-comments))
  (font-lock-fontify-buffer))

(defvar foolang--reserved-words
  '(
    "class"
    "define"
    "end"
    "extend"
    "import"
    "interface"
    "is"
    "let"
    "method"
    "raise"
    "required"
    "return"
    ))

(font-lock-add-keywords
 'foolang-mode
 (append
    ;; Foolang keywords from the list above. Do NOT use \\> to match word end as
  ;; that would match keywords arguments.
  (mapcar (lambda (keyword)
            (cons (format "\\<%s\\($\\|\\s-+\\)" keyword) 'font-lock-keyword-face))
          foolang--reserved-words)
  ;; Keyword arguments in method definitions and calls
  '(("\\<method\\s-+\\([^ :]+\\)[ :]" 1 font-lock-function-name-face)
    ("\\<\\w+:[^:]" . font-lock-function-name-face))
  ;; Contexts in which type names appear as type names instead of
  ;; just plain vanilla objects.
  '(("\\<class\\s-+\\(\\w+\\)\\>" 1 font-lock-type-face)
    ("\\<extend\\s-+\\(\\w+\\)\\>" 1 font-lock-type-face)
    ("\\<interface\\s-+\\(\\w+\\)\\>" 1 font-lock-type-face)
    ("::\\(\\w+\\)\\>" 1 font-lock-type-face))
  ;; Variable binding and assignment.
  '(("\\<let\\s-+\\(\\w+\\)\\>" 1 font-lock-variable-name-face)
    ("\\<\\(\\w+\\)\\s-*=[^=]" 1 font-lock-variable-name-face))))

(add-to-list 'auto-mode-alist '("\\.foo" . foolang-mode))

(defvar foolang-indent-offset 4)

(defvar foolang--indent-rules)
(setq foolang--indent-rules nil)

(cl-defmacro def-foolang-indent (name (col base stack ctx) &body body)
  `(let ((rule (list
                (lambda (,ctx) ,@(cdr (assoc :after body)))
                (lambda (,col ,base ,stack ,ctx) ,@(cdr (assoc :indent body)))))
        (old (assoc-string ,name foolang--indent-rules)))
     (if old
         (setcdr old rule)
       (push (cons ,name rule) foolang--indent-rules))))

(defconst foolang--newline-or-lines-regex
  "\\(\n\\s-*\\)+")

(defconst foolang--space-or-eol-regex
  "\\(\\s-+|\\)")

(defconst foolang--method-or-class-method-regex
  "\\(class\\s-+\\)?method\\s-+")

(defconst foolang--op-regex
  "\\s_+\\s-+")

(defconst foolang--name-with-opt-type-regex
  "\\w+\\(::\\w+\\)?\\>\\s-*")

(defconst foolang--keyword-regex
  "\\w+:[^:]\\s-*")

(defconst foolang--keywords-and-names-with-opt-types-regex
  (concat "\\("
          foolang--keyword-regex
          foolang--name-with-opt-type-regex
          "\\)+"))

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

(def-foolang-indent "interface" (col base stack ctx)
  (:after
    (looking-at " *interface\\s-*$"))
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

(def-foolang-indent "class Name {...}" (col base stack ctx)
  (:after
    (looking-at " *class\\s-+[A-Z]+\\s-*{.*}\\s-*$"))
  (:indent
   (list (+ col foolang-indent-offset)
         base
         stack
         ctx)))

(def-foolang-indent "define <name>" (col base stack ctx)
  (:after
    (looking-at " *define\\s-+[A-Za-z_]+\\s-*$"))
  (:indent
   (list (+ col foolang-indent-offset)
         base
         stack
         ctx)))

(def-foolang-indent "extend Name" (col base stack ctx)
  (:after
    (looking-at " *extend\\s-+[A-Z]+\\s-*$"))
  (:indent
   (list (+ col foolang-indent-offset)
         base
         stack
         ctx)))

(def-foolang-indent "interface Name" (col base stack ctx)
  (:after
    (looking-at " *interface\\s-+[A-Z]+\\s-*$"))
  (:indent
   (list (+ col foolang-indent-offset)
         base
         stack
         ctx)))

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

(def-foolang-indent "expr exit list." (col base stack ctx)
  (:after
    (and (looking-at ".*\\w")
         (foolang--nesting-decreases-on-line)
         (foolang--looking-at-terminated-line)))
  (:indent
   (beginning-of-line)
   (let (top)
     (while (foolang--exit-list-on-current-line)
       (assert stack nil "expr exit list. => stack empty")
       (setq top (pop stack)))
     (list (cdr top) (cdr top) stack ctx))))

(def-foolang-indent "exit keyword list \\ name:" (col base stack ctx)
  (:after
    (and (looking-at ".*\\w")
         (foolang--exit-from-keyword-argument-list)
         (save-excursion
           (end-of-line)
           (looking-at (concat foolang--newline-or-lines-regex
                               foolang--keyword-regex)))))
  (:indent
   ()
   (beginning-of-line)
   (let (top)
     (while (foolang--exit-list-on-current-line)
       (assert stack nil "expr exit list. => stack empty")
       (setq top (pop stack)))
     (list (foolang--exit-from-keyword-argument-list) (cdr top) stack ctx))))

(def-foolang-indent "expr exit list" (col base stack ctx)
  (:after
    (and (looking-at ".*\\w")
         (foolang--nesting-decreases-on-line)))
  (:indent
   (beginning-of-line)
   (let (top)
     (while (foolang--exit-list-on-current-line)
       (assert stack nil "expr exit list. => stack empty")
       (setq top (pop stack)))
     (list (car top) (cdr top) stack ctx))))

(def-foolang-indent "expr \\ exit list" (col base stack ctx)
  (:after
    (save-excursion
      (when (not (foolang--looking-at-terminated-line))
        (next-line)
        (beginning-of-line)
        (looking-at " *[]})]"))))
  (:indent
   (let ((top (car stack)))
     (list (car top) (cdr top) (cdr stack) ctx))))

(def-foolang-indent "enter list expr. or |...|" (col base stack ctx)
  (:after
    (and (foolang--nesting-increases-on-line)
         (or (foolang--looking-at-terminated-line)
             (save-excursion
               (foolang--end-of-code-on-line)
               (looking-at "|")))))
  (:indent
   (end-of-line)
   (backward-up-list)
   (let ((p (current-column)))
     (list (+ p 2) (+ p 2)
           (cons (cons col base) stack)
           ctx))))

(def-foolang-indent "enter list expr name: expr \\ name:" (col base stack ctx)
  (:after
    (and (foolang--nesting-increases-on-line)
         (not (foolang--looking-at-terminated-line))
         (save-excursion
           (when (condition-case nil (progn (down-list) t))
             (looking-at ".+:\\s-*[^\n:]+\n\\(\n\\|\\s-\\)*\\w+:")))))
  (:indent
   (down-list)
   (let ((p (current-column)))
     (search-forward ":")
     (backward-word)
     (let ((q (current-column)))
       (list q
             (+ p 2)
             (cons (cons col base) stack)
             ctx)))))

(def-foolang-indent "enter list \\" (col base stack ctx)
  (:after
    (save-excursion
      (foolang--end-of-code-on-line)
      (looking-at "[[{(]")))
  (:indent
   (list (+ col foolang-indent-offset)
         (+ col foolang-indent-offset)
         (cons (cons col base) stack)
         ctx)))

(def-foolang-indent "enter list let name = expr" (col base stack ctx)
  (:after
    (when (eq :body ctx)
      (and (foolang--nesting-increases-on-line)
           (not (foolang--looking-at-terminated-line))
           (looking-at ".*let\\s-+\\w+\\s-*=\\s-*\\S-+"))))
  (:indent
   (end-of-line)
   (search-backward "=")
   (let ((p (current-column)))
     (list (+ (current-column) 2 foolang-indent-offset) base stack ctx))))

(def-foolang-indent "enter list expr" (col base stack ctx)
  (:after
    (and (foolang--nesting-increases-on-line)
         (not (foolang--looking-at-terminated-line))))
  (:indent
   (end-of-line)
   (backward-up-list)
   (let ((p (current-column)))
     (list (+ p 2 foolang-indent-offset)
           (+ p 2)
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

(defconst foolang--method-op-arg-with-opt-type-regex
  (concat "\\s-*"
          foolang--method-or-class-method-regex
          foolang--op-regex
          foolang--name-with-opt-type-regex
          "$"))

(def-foolang-indent "method op arg" (col base stack ctx)
  (:after
    (looking-at foolang--method-op-arg-with-opt-type-regex))
  (:indent
   (list (+ col foolang-indent-offset)
         (+ col foolang-indent-offset)
         stack
         :body)))

(defconst foolang--method-keywords-across-lines-regex
  (concat "\\s-*"
          foolang--method-or-class-method-regex
          foolang--keywords-and-names-with-opt-types-regex
          foolang--newline-or-lines-regex
          foolang--keyword-regex
          ;; No eol check here, since we don't care what the
          ;; next line contains, just that is _starts_ with
          ;; a keyword.
          ))

(def-foolang-indent "method name: arg \\ name:" (col base stack ctx)
  (:after
    (looking-at foolang--method-keywords-across-lines-regex))
  (:indent
   (search-forward ":")
   (backward-word)
   (let ((p (current-column)))
     (list p
           (+ col foolang-indent-offset)
           stack
           :method))))

(defconst foolang--keywords-across-lines-regex
  (concat "\\s-*"
          foolang--keywords-and-names-with-opt-types-regex
          foolang--newline-or-lines-regex
          foolang--keyword-regex
          ;; No eol check here, since we don't care what the
          ;; next line contains, just that is _starts_ with
          ;; a keyword.
          ))

(def-foolang-indent "tailing method name: arg \\ name:" (col base stack ctx)
  (:after
    (when (eq :method ctx)
      (looking-at foolang--keywords-across-lines-regex)))
  (:indent
   (list col base stack ctx)))

(def-foolang-indent "tailing method fallthrough" (col base stack ctx)
  (:after
    (eq :method ctx))
  (:indent
   (list base base stack :body)))

(defconst foolang--method-keywords-regex
  (concat "\\s-*"
          foolang--method-or-class-method-regex
          foolang--keywords-and-names-with-opt-types-regex
          "$"))

(def-foolang-indent "method name: arg" (col base stack ctx)
  (:after
    (looking-at foolang--method-keywords-regex))
  (:indent
   (list (+ col foolang-indent-offset)
         (+ col foolang-indent-offset)
         stack
         :body)))

(def-foolang-indent "in-body expr name: expr \\ name:" (col base stack ctx)
  (:after
    (when (eq :body ctx)
      (and (not (foolang--looking-at-terminated-line))
           (foolang--looking-at-keyword-message-eol)
           (save-excursion
             (next-line)
             (foolang--looking-at-keyword-message-bol)))))
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
      (and (not (foolang--looking-at-terminated-line))
           (looking-at "\\s-*;[^\n]*\n\\s-*;"))))
  (:indent
   (list col base stack ctx)))

(def-foolang-indent "let name = expr" (col base stack ctx)
  (:after
    (when (eq :body ctx)
      (and (not (foolang--looking-at-terminated-line))
           (looking-at ".*let\\s-+\\w+\\s-*=\\s-*\\S-+"))))
  (:indent
   (search-forward "=")
   (let ((p (current-column)))
     (list (+ (current-column) 1 foolang-indent-offset) base stack ctx))))

(def-foolang-indent "expr" (col base stack ctx)
  (:after
    (when (eq :body ctx)
      (not (foolang--looking-at-terminated-line))))
  (:indent
   (list (+ col foolang-indent-offset) base stack ctx)))

(def-foolang-indent "{ expr." (col base stack ctx)
  (:after
    (when (eq :body ctx)
      (and (foolang--looking-at-terminated-line)
           (foolang--nesting-increases-on-line))))
  (:indent
   (end-of-line)
   (backward-up-list)
   (let ((p (+ 2 (current-column))))
     (list p p (cons (cons col base) stack) ctx))))

(def-foolang-indent "expr." (col base stack ctx)
  (:after
    (when (eq :body ctx)
      (foolang--looking-at-terminated-line)))
  (:indent
   (list base base stack ctx)))

;;;; Indentation engine

;; Like this so that I can just C-x C-e here to turn it on.
(defvar foolang--debug-indentation)
(setq foolang--debug-indentation t)
(setq foolang--debug-indentation nil)

(defun foolang--note (control &rest args)
  (when foolang--debug-indentation
    (with-current-buffer (get-buffer-create "*foolang-indentation*")
      (end-of-buffer)
      (insert (apply #'format control args))
      (insert "\n"))))

(defun foolang-indent-line ()
  (interactive)
  (foolang--indent-line-number (line-number-at-pos) nil))

(defun foolang-indent-all ()
  (interactive)
  (foolang--indent-line-number (line-number-at-pos) t))

(defun foolang--indent-line-number (line-number indent-all)
  (let ((line-move-visual nil))
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
    (cond ((eql target now)
           (indent-line-to col))
          ((looking-at "^\\s-*$")
           ;; Skip over empty lines
           (next-line)
           (beginning-of-line)
           (foolang--indent-to target col base stack ctx indent-all))
          (t
           ;; Compute indentation for next line and move down.
           (when indent-all
             (indent-line-to col))
           (destructuring-bind (new-col new-base new-stack new-ctx)
               (foolang--compute-next-line-indent col base stack ctx)
             (next-line)
             (beginning-of-line)
             (foolang--indent-to target new-col new-base new-stack new-ctx indent-all))))))

(cl-defun foolang--compute-next-line-indent (col base stack ctx)
  (foolang--note "indent: '%s'" (foolang--current-line))
  (dolist (rule (reverse foolang--indent-rules))
    (beginning-of-line)
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
        ((foolang--looking-at-define) 0)
        ((foolang--looking-at-extend) 0)
        ((foolang--looking-at-interface) 0)
        ((foolang--top-of-buffer) 0)))

(defun foolang--looking-at-method ()
  (looking-at " *\\(method\\|class *method\\)"))

(defun foolang--looking-at-class ()
  (looking-at " *class [A-Z]"))

(defun foolang--looking-at-define ()
  (looking-at " *define [A-Za-z_]"))

(defun foolang--looking-at-extend ()
  (looking-at " *extend [A-Z]"))

(defun foolang--looking-at-interface ()
  (looking-at " *interface [A-Z]"))

(defun foolang--top-of-buffer ()
  (eql 1 (line-number-at-pos)))

(defun foolang--end-of-code-on-line ()
  (beginning-of-line)
  (let ((end (re-search-forward "--" (line-end-position) t)))
    (if end
        (goto-char (- end 3))
      (end-of-line)
      (when (and (looking-at "\n") (> (current-column) 0))
        (backward-char))))
  (while (looking-at "\\s-")
    (backward-char)))

(defun foolang--looking-at-terminated-line ()
  (save-excursion
    (foolang--end-of-code-on-line)
    (looking-at "[\\.,]")))

(defun foolang--exit-from-keyword-argument-list ()
  (and (foolang--nesting-decreases-on-line)
       (save-excursion
         (beginning-of-line)
         (backward-up-list)
         (backward-char)
         (foolang--skip-whitespace-left)
         (when (looking-at ":")
           (backward-word)
           (current-column)))))

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

(defun foolang--exit-list-on-current-line ()
  (let* ((line (line-number-at-pos))
         (p (condition-case nil
                (save-excursion
                  (backward-up-list)
                  (forward-list)
                  (when (eql line (line-number-at-pos))
                    (point)))
              (error nil))))
    (when p
      (goto-char p))))

(defun foolang--skip-whitespace-left ()
  (while (looking-at "[\n\t ]")
    (backward-char)))

(defun foolang--backward-expr ()
  (interactive)
  (foolang--skip-whitespace-left)
  (while (looking-at ":")
    (backward-char))
  (cond ((looking-at "[]})]")
         (backward-up-list)
         (backward-char))
        ((looking-at "\\w")
         (while (looking-at "\\w")
           (backward-char)))
        ((looking-at "\\s_")
         (while (looking-at "\\s_")
           (backward-char)))
        ((and (looking-at "\"") (not (char-equal ?\ (char-before))))
         ;; KLUDGE: Need to move out of the quote before I can do backward-sexp
         ;; ...clearly the way I structured this is a bad fit for rest of emacs.
         ;; ...FIXME.
         (forward-char)
         (backward-sexp)
         (backward-char)))
  (foolang--skip-whitespace-left))

(cl-defun foolang--looking-at-keyword-message-eol ()
  (save-excursion
    (foolang--end-of-code-on-line)
    (unless (looking-at ":")
      (let ((line (line-number-at-pos))
            (p nil))
        (while (and (eql line (line-number-at-pos))
                    (not (looking-at ":"))
                    (not (eql p (point))))
          (setq p (point))
          (foolang--backward-expr))
        (and (eql line (line-number-at-pos))
             (looking-at ":"))))))

(defun foolang--looking-at-keyword-message-bol ()
  (save-excursion
    (beginning-of-line)
    (looking-at "\\s-*\\w+:")))

;;;; Indentation testing

(defvar foolang--indentation-tests)
(setq foolang--indentation-tests nil)

(defvar foolang--indentation-test-failures)
(setq foolang--indentation-test-failures nil)

(defun foolang--push-mark-around (old &optional loc nomsg activate)
  (funcall old loc t activate))

(defun foolang--run-indentation-test (name source target)
  (with-current-buffer (get-buffer-create "*foolang-indentation*")
    (end-of-buffer)
    (let ((start (point)))
      (insert "\n--- " name " ---\n")
      (lexical-let ((result (with-temp-buffer
                              (foolang-mode)
                              (setq indent-tabs-mode nil)
                              (insert source)
                              (foolang-indent-all)
                              (end-of-buffer)
                              (buffer-substring-no-properties 1 (point)))))
        (end-of-buffer)
        (if (string= target result)
            (progn (insert "ok!")
                   (message "test %s ok" name))
          (push name foolang--indentation-test-failures)
          (insert "FAILED!\n")
          (insert "WANTED:\n")
          (insert target)
          (insert "\nGOT:\n")
          (insert result)
          (lexical-let ((same nil))
            (condition-case nil
                (dotimes (i (length target))
                  (if (string= (substring-no-properties target 0 i)
                               (substring-no-properties result 0 i))
                      (setq same i)
                    (insert (format "\nFAIL: %S" i))
                    (insert (format "\nDifference at %s: %S vs %S\n"
                                    i (aref target (- i 1)) (aref result (- i 1))))
                    (return)))
              (error nil))
            (insert (format "\nSame until char %s, target len=%s, result len=%s\n"
                            same (length target) (length result)))
            (when (and same (> same 0))
              (insert "Identical part:\n")
              (insert (substring-no-properties target 0 same))))
          (message "test %s FAILED:\n%s" name
                   (buffer-substring-no-properties start (point))))))))

(defun foolang--run-tests ()
  ;; For now these are all indentation tests. When others are
  ;; added this should be split into parts.
  (with-current-buffer (get-buffer-create "*foolang-indentation*")
    (erase-buffer))
  (setq foolang--indentation-test-failures nil)
  (advice-add 'push-mark :around 'foolang--push-mark-around)
  (unwind-protect
      (dolist (test foolang--indentation-tests)
        (apply 'foolang--run-indentation-test test))
    (advice-remove 'push-mark 'foolang--push-mark-around))
  (with-current-buffer "*foolang-indentation*"
    (cond (foolang--indentation-test-failures
           (display-buffer "*foolang-indentation*")
           (user-error "Foolang indentation tests failed!"))
          (t
           (kill-buffer)
           (message "Foolang tests ok!")))))

(cl-defmacro def-foolang-indent-test (name source target)
  `(let* ((name ,name)
         (test (list ,source ,target))
         (old (assoc-string name foolang--indentation-tests)))
     (if old
         (setcdr old test)
       (push (cons name test) foolang--indentation-tests))
     (when foolang--debug-indentation
       (apply 'foolang--run-indentation-test name test))))

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

(def-foolang-indent-test "class-indent-7"
  "
class Foo { a }
is"
  "
class Foo { a }
    is")

(def-foolang-indent-test "interface-indent-1"
  "
interface Foo
method bar"
  "
interface Foo
    method bar")

(def-foolang-indent-test "extend-indent-1"
  "
extend Foo
method bar"
  "
extend Foo
    method bar")

(def-foolang-indent-test "extend-indent-2"
  "
extend Foo
class method bar"
  "
extend Foo
    class method bar")

(def-foolang-indent-test "extend-indent-3"
  "
extend Foo
is Bar"
  "
extend Foo
    is Bar")

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

(def-foolang-indent-test "method-indent-3.1"
  "
class Foo { a }
method bar: x::Type
quux: y::Type
x + y"
  "
class Foo { a }
    method bar: x::Type
           quux: y::Type
        x + y")

(def-foolang-indent-test "method-indent-3.2"
  "
class Foo { a }
method bar: x::Type
quux: y::Type
zo: z::Type
- 42 + x + y + z"
  "
class Foo { a }
    method bar: x::Type
           quux: y::Type
           zo: z::Type
        - 42 + x + y + z")

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

(def-foolang-indent-test "method-indent-6"
  "
class Main {}
class method + x::Type
x"
  "
class Main {}
    class method + x::Type
        x")

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

(def-foolang-indent-test "body-indent-14"
  "
method check: cond on: x onSuccess: success onFailure: failure
let res = { cond value: x }
onError: { |e ctx|
system output println: \"ERROR: {e}\".
failure value }.
res
ifTrue: success
ifFalse: failure"
  "
    method check: cond on: x onSuccess: success onFailure: failure
        let res = { cond value: x }
                      onError: { |e ctx|
                                 system output println: \"ERROR: {e}\".
                                 failure value }.
        res
            ifTrue: success
            ifFalse: failure")

(def-foolang-indent-test "body-indent-15"
  "
method forAll: generator that: cond testing: thing
let n = 0.
generator
do: { |x| self check: cond on: x
onSuccess: { n = n + 1 }
onFailure: { system output
println: \"! {thing} failed on: {x}\".
fail = True.
return False }}.
system output println: \"  {thing} ok ({n} assertions)\".
True"
  "
    method forAll: generator that: cond testing: thing
        let n = 0.
        generator
            do: { |x| self check: cond on: x
                           onSuccess: { n = n + 1 }
                           onFailure: { system output
                                            println: \"! {thing} failed on: {x}\".
                                        fail = True.
                                        return False }}.
        system output println: \"  {thing} ok ({n} assertions)\".
        True")

(def-foolang-indent-test "body-indent-16"
  "
method testFilePath
let pathX = files path: \"X\".
assert raises: { files path: \"..\" }
error: \"Cannot extend #<FilePath (root)> with ..\"
testing:"
  "
    method testFilePath
        let pathX = files path: \"X\".
        assert raises: { files path: \"..\" }
               error: \"Cannot extend #<FilePath (root)> with ..\"
               testing:")

(def-foolang-indent-test "body-indent-17"
  "
method foo
bar.

quux."
  "
    method foo
        bar.

        quux.")

(def-foolang-indent-test "body-indent-18"
  "
method foo
x run: { let x = y bar
quux."
  "
    method foo
        x run: { let x = y bar
                             quux.")

(def-foolang-indent-test "body-indent-19"
  "
method foo
x run: { bing boing.
let x = y bar
quux."
  "
    method foo
        x run: { bing boing.
                 let x = y bar
                             quux.")

(def-foolang-indent-test "body-indent-20"
  "
method testRecord
assert true: { let r = {x: -10, y: 52}.
r x + r y == 42 }
testing: \"record creation and accessors\""
  "
    method testRecord
        assert true: { let r = {x: -10, y: 52}.
                       r x + r y == 42 }
               testing: \"record creation and accessors\"")

(def-foolang-indent-test "end-indent-1"
  "
method bar
42
end"
  "
    method bar
        42
end")

(def-foolang-indent-test "define-indent-1"
  "
define foo
42
end"
  "
define foo
    42
end")

(foolang--run-tests)

(provide 'foolang)
