;;; c:/Users/nikod/src/foolang/foolang.el -*- lexical-binding: t; -*-

(require 'cl)

;; Soft require: ./test-elisp.sh provides this, but we don't depend on it
(require 'highlight-numbers nil t)

(defvar foolang-syntax-table (make-syntax-table))
;; (setq foolang-syntax-table (make-syntax-table))

(defun foolang-init-syntax-table (table)
  (modify-syntax-entry ?_ "w" table)
  (modify-syntax-entry ?\" "\"" table)
  (modify-syntax-entry ?\; "." table)
  (modify-syntax-entry ?^ "_" table)
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
   ;; A lone '-' is a symbol.
   ("\\<\\(-\\)[^-0-9]" (1 "_"))
   ;; A leading '-' before a number is punctuation.
   ("[^-]\\(-\\)[0-9]" (1 "."))
   ("^\\(-\\)[0-9]" (1 "."))
   ;; Block comments as generic fences.
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
  (when (fboundp 'highlight-numbers-mode)
    (highlight-numbers-mode))
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
    "direct"
    ))

(font-lock-add-keywords
 'foolang-mode
 (append
    ;; Foolang keywords from the list above. Do NOT use \\> to match word end as
  ;; that would match keywords arguments.
  (mapcar (lambda (keyword)
            (cons (format "\\<%s\\($\\|\\s-+\\)" keyword) 'font-lock-keyword-face))
          foolang--reserved-words)
  ;; Method names in definitions
  '(("\\<method\\s-+\\([^ :]+\\)" 1 font-lock-function-name-face))
  ;; Keyword arguments
  '(("\\<\\(\\w+:\\)[^:]" 1 font-lock-function-name-face))
  ;; Contexts in which type names appear as type names instead of
  ;; just plain vanilla objects.
  '(("\\<class\\s-+\\(\\w+\\)\\>" 1 font-lock-type-face)
    ("\\<extend\\s-+\\(\\w+\\)\\>" 1 font-lock-type-face)
    ("\\<interface\\s-+\\(\\w+\\)\\>" 1 font-lock-type-face)
    ("::\\(\\w+\\)\\>" 1 font-lock-type-face))
  ;; Constant definition, variable binding and assignment.
  '(("\\<define\\s-+\\(\\w+\\)\\>" 1 font-lock-variable-name-face)
    ("\\<let\\s-+\\(\\w+\\)\\>" 1 font-lock-variable-name-face)
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

(defconst foolang--eol-regex
  "\\s-*$")

(defconst foolang--method-or-direct-method-regex
  "\\(direct\\s-+\\)?method\\s-+")

(defconst foolang--name-regex
  "\\w+")

(defconst foolang--op-regex
  "\\s_+\\s-+")

(defconst foolang--name-with-opt-type-regex
  (concat foolang--name-regex
          "\\(::\\s-*\\w+\\)?\\>\\s-*"))

(defconst foolang--keyword-regex
  "\\w+:[^:]\\s-*")

(defconst foolang--keywords-and-names-with-opt-types-regex
  (concat "\\("
          foolang--keyword-regex
          foolang--name-with-opt-type-regex
          "\\)+"))

(defconst foolang--class-and-name-regex
  "\\s-*class\\s-+\\w+")

(defconst foolang--open-brace-regex
  "\\s-*{")

(defconst foolang--class-and-name-and-open-brace-regex
  (concat foolang--class-and-name-regex
          foolang--open-brace-regex
          foolang--eol-regex))

(defconst foolang--class-and-name-and-open-brace-and-slotnames-regex
  (concat foolang--class-and-name-regex
          foolang--open-brace-regex
          "\\(\\s-+" foolang--name-with-opt-type-regex "\\)+"
          foolang--eol-regex))

(defconst foolang--slots-regex
  ;; FIXME: sloppy regex
  (concat "\\s-*{.*}"))

(defconst foolang--class-and-name-and-slots-and-newline-end-regex
  (concat  foolang--class-and-name-regex
           foolang--slots-regex
           foolang--newline-or-lines-regex
           "end\\s-*$"))

(defconst foolang--class-and-name-and-slots-regex
  (concat foolang--class-and-name-regex
          foolang--slots-regex
          "\\s-*$"))

(defconst foolang--comment-regex
  "\\s-*--")

(def-foolang-indent "-- comment" (col base stack ctx)
  (:after
    (looking-at foolang--comment-regex))
  (:indent
   (list col base stack ctx)))

(def-foolang-indent "end" (col base stack ctx)
  (:after
    (looking-at "\\s-*end\\>"))
  (:indent
   (list 0 0 nil :toplevel)))

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
    (looking-at foolang--class-and-name-and-open-brace-regex))
  (:indent
   (list (* 2 foolang-indent-offset)
         (* 2 foolang-indent-offset)
         (cons (cons foolang-indent-offset foolang-indent-offset)
               stack)
         :slots)))

(def-foolang-indent "class Name { slot" (col base stack ctx)
  (:after
    (looking-at foolang--class-and-name-and-open-brace-and-slotnames-regex))
  (:indent
   (search-forward "{")
   (let ((p (foolang--line-length-to-here)))
     (list (+ p 1) (+ p 1)
           (cons (cons foolang-indent-offset foolang-indent-offset)
                 stack)
           :slots))))

(def-foolang-indent "class Name {...} \\ end" (col base stack ctx)
  (:after
    (looking-at foolang--class-and-name-and-slots-and-newline-end-regex))
  (:indent
   (list col
         base
         stack
         ctx)))

(def-foolang-indent "class Name {...}" (col base stack ctx)
  (:after
    (looking-at foolang--class-and-name-and-slots-regex))
  (:indent
   (list (+ col foolang-indent-offset)
         base
         stack
         ctx)))

(def-foolang-indent "define <name>" (col base stack ctx)
  (:after
    (looking-at " *define\\s-+\\w++\\s-*$"))
  (:indent
   (list (+ col foolang-indent-offset)
         base
         stack
         ctx)))

(def-foolang-indent "extend Name" (col base stack ctx)
  (:after
    (looking-at " *extend\\s-+\\w+\\s-*$"))
  (:indent
   (list (+ col foolang-indent-offset)
         base
         stack
         ctx)))

(def-foolang-indent "interface Name" (col base stack ctx)
  (:after
    (looking-at " *interface\\s-+\\w+\\s-*$"))
  (:indent
   (list (+ col foolang-indent-offset)
         base
         stack
         ctx)))

(defconst foolang--line-followed-by-end-regex
  (concat "[^\n]*"
          foolang--newline-or-lines-regex
          "\\s-*end\\>"))

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

(def-foolang-indent "enter list(s) expr. or |...|" (col base stack ctx)
  (:after
    (and (foolang--nesting-increases-on-line)
         (or (foolang--looking-at-terminated-line)
             (save-excursion
               (foolang--end-of-code-on-line)
               (looking-at "|")))))
  (:indent
   (let ((n (foolang--nesting-increases-on-line)))
     (end-of-line)
     (backward-up-list n)
     (loop repeat n do
           (down-list)
           (foolang--skip-horizontal-whitespace-right)
           (push (cons col base) stack)
           (setf col (current-column)
                 base col))
     (list col base stack ctx))))

(def-foolang-indent "enter list expr name: expr \\ name:" (col base stack ctx)
  (:after
    (and (foolang--nesting-increases-on-line)
         (not (foolang--looking-at-terminated-line))
         (save-excursion
           (when (condition-case nil (progn (down-list) t))
             (when (looking-at ".+\\(:\\)\\s-*[^\n:]+\n\\(\n\\|\\s-\\)*\\w+:")
               (backward-up-list)
               (let ((p (point)))
                 (goto-char (match-beginning 1))
                 (backward-up-list)
                 (eql p (point))))))))
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

(def-foolang-indent "enter list(s) \\" (col base stack ctx)
  (:after
    (when (eq :body ctx)
      (save-excursion
        (foolang--end-of-code-on-line)
        (looking-at "[[{(]"))))
  (:indent
   (let ((n (foolang--nesting-increases-on-line)))
     (end-of-line)
     (backward-up-list n)
       (dotimes (_ n)
         (down-list)
         (let ((indent (if (looking-at "\\s-*$")
                           (+ col foolang-indent-offset)
                         (foolang--skip-horizontal-whitespace-right)
                         (current-column))))
         (push (cons col base) stack)
         (setf col indent
               base indent)))
     (list col base stack ctx))))

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

(def-foolang-indent "enter list(s) expr" (col base stack ctx)
  (:after
    (and (foolang--nesting-increases-on-line)
         (not (foolang--looking-at-terminated-line))))
  (:indent
   (let ((n (foolang--nesting-increases-on-line)))
     (end-of-line)
     (backward-up-list n)
     (loop repeat n do
           (down-list)
           (foolang--skip-horizontal-whitespace-right)
           (push (cons col base) stack)
           (setf col (current-column)
                 base col))
     (list (+ col foolang-indent-offset) base stack ctx))))

(defconst foolang--method-op-arg-with-opt-type-regex
  (concat "\\s-*"
          foolang--method-or-direct-method-regex
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
          foolang--method-or-direct-method-regex
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

(defconst foolang--method-unary-regex
  (concat "\\s-*"
          foolang--method-or-direct-method-regex
          foolang--name-regex))

(def-foolang-indent "method name" (col base stack ctx)
  (:after
    (looking-at foolang--method-unary-regex))
  (:indent
   (list (+ col foolang-indent-offset)
         (+ col foolang-indent-offset)
         stack
         :body)))

(defconst foolang--method-keywords-regex
  (concat "\\s-*"
          foolang--method-or-direct-method-regex
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
             (while (looking-at foolang--comment-regex)
               (next-line))
             (foolang--looking-at-keyword-message-bol)))))
  (:indent
   (search-forward ":")
   (backward-word)
   (let ((p (current-column)))
     (list p base stack ctx))))

(defconst foolang--optional-horizontal-whitespace-regex
  "\\s-*")

(defconst foolang--regex-to-newline "[^\n]*")

(defconst foolang--keyword-arg-line-regex
  (concat foolang--optional-horizontal-whitespace-regex
          foolang--keyword-regex foolang--regex-to-newline
          foolang--newline-or-lines-regex))

(defconst foolang--two-keyword-arg-lines-regex
  (concat foolang--keyword-arg-line-regex
          foolang--optional-horizontal-whitespace-regex
          foolang--keyword-regex))

(def-foolang-indent "in-body name: var \\ name: var" (col base stack ctx)
  (:after
    (when (eq :body ctx)
      (looking-at foolang--two-keyword-arg-lines-regex)))
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

(def-foolang-indent "expr \ enter block" (col base stack ctx)
  (:after
    (when (eq :body ctx)
      (and (not (foolang--looking-at-terminated-line))
           (save-excursion
             (next-line)
             (looking-at "\\s-*{")))))
  (:indent
   (list col base stack ctx)))

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

;; Like this so that I can just C-x C-e here to turn it on / off,
;; and it sticks on reload.
(defvar foolang--debug-mode nil)
;; (setq foolang--debug-mode t)
;; (setq foolang--debug-mode nil)

(defun foolang--note (control &rest args)
  (when foolang--debug-mode
    (with-current-buffer (get-buffer-create "*foolang-notes*")
      (end-of-buffer)
      (insert (apply #'format control args))
      (insert "\n"))))

(defmacro foolang--save-indentation (body)
  `(prog1
       (save-excursion ,body)
     (let ((region-start (line-beginning-position)))
       (when (looking-back "^\\s-*")
         (back-to-indentation)
         (setq region-start (point)))
       (when (looking-at "\\s-*$")
         (delete-trailing-whitespace region-start (line-end-position))))))

(defun foolang-indent-line ()
  (interactive)
  (foolang--save-indentation
    (foolang--indent-line-number (line-number-at-pos) nil)))

(defun foolang-indent-all ()
  (interactive)
  (foolang--save-indentation
    (foolang--indent-line-number (line-number-at-pos) t)))

(defun foolang--indent-line-number (line-number indent-all)
  (let ((line-move-visual nil)
        (case-fold-search nil))
    (let ((base (foolang--find-indent-base indent-all)))
      (foolang--indent-to line-number base base nil nil indent-all))))

(defun foolang--find-indent-base (indent-all)
  "Search lines up until it find a 'base', meaning
   a class or method definition line, or top of buffer."
  (lexical-let ((base nil))
    (while (not (setq base (foolang--indent-base-or-nil indent-all)))
      (previous-line))
    (foolang--note "indent-base(%s): %s at '%s'"
                   (if indent-all "all" "local")
                   base
                   (foolang--current-line))
    base))

(defun foolang--indent-line-to (col)
  (if (looking-at "\\s-*end\\>")
      (indent-line-to 0)
    (indent-line-to col)))

(defun foolang--dont-indent ()
  (let ((s (syntax-ppss)))
    (or (nth 3 s) ; inside string
        (and (nth 4 s) ; inside comment, but not final ---
             (not (looking-at "---"))))))

(defun foolang--indent-to (target col base stack ctx indent-all)
  (let ((now (line-number-at-pos)))
    ;; (foolang--note "line %s (target %s)" now target)
    (cond ((eql target now)
           (unless (foolang--dont-indent)
             (foolang--indent-line-to col)))
          ((or (looking-at "^\\s-*$") (foolang--dont-indent))
           ;; Skip over empty lines and strings and comments
           (next-line)
           (beginning-of-line)
           (foolang--indent-to target col base stack ctx indent-all))
          (t
           ;; Compute indentation for next line and move down.
           (when indent-all
             (foolang--indent-line-to col))
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
      ; (foolang--note "  not '%s'" (first rule))
      ))
  (foolang--note "  ! No rule to indent line after '%s'" (foolang--current-line))
  (list col base stack ctx))

(defun foolang--indent-base-or-nil (indent-all)
  (beginning-of-line)
  (cond ((foolang--dont-indent)
         nil)
        ((foolang--looking-at-method)
         (if indent-all
             nil
           (foolang--note "indent-base is method")
           foolang-indent-offset))
        ((foolang--looking-at-class)
         (foolang--note "indent-base is class")
         0)
        ((foolang--looking-at-define)
         (foolang--note "indent-base is define")
         0)
        ((foolang--looking-at-extend)
         (foolang--note "indent-base is extend")
         0)
        ((foolang--looking-at-interface)
         (foolang--note "indent-base is interface")
         0)
        ((foolang--top-of-buffer)
         (foolang--note "indent-base is top-of-buffer")
         0)))

(defun foolang--looking-at-method ()
  (looking-at " *\\(method\\|direct *method\\)\\>"))

(defun foolang--looking-at-class ()
  (looking-at " *class [A-Za-z_]"))

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
    (let ((n 0))
      (while (eql (line-number-at-pos)
                  (condition-case nil
                      (progn (backward-up-list)
                             (forward-list)
                             (line-number-at-pos))
                    (error nil)))
        (incf n))
      (when (> n 0)
        n))))

(defun foolang--nesting-increases-on-line ()
  (save-excursion
    (end-of-line)
    (let ((n 0))
      (while (eql (line-number-at-pos)
                  (condition-case nil
                      (progn (backward-up-list)
                             (line-number-at-pos))
                    (error nil)))
        (incf n))
      (when (> n 0)
        n))))

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

(defun foolang--skip-horizontal-whitespace-right ()
  (while (looking-at "\\s-")
    (forward-char)))

(defun foolang--backward-expr ()
  (interactive)
  (foolang--skip-whitespace-left)
  (while (looking-at ":")
    (backward-char))
  (cond ((and (looking-at "[]})]") (ignore-errors (backward-up-list) t))
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

(defvar foolang--test-failures)
(setq foolang--test-failures nil)

(defun foolang--push-mark-around (old &optional loc nomsg activate)
  (funcall old loc t activate))

(defun foolang--run-indentation-test (name source target)
  (with-current-buffer (get-buffer-create "*foolang-notes*")
    (end-of-buffer)
    (let ((start (point)))
      (insert "\n--- " name " ---\n")
      (lexical-let ((result (with-temp-buffer
                              (foolang-mode)
                              (setq indent-tabs-mode nil)
                              (insert source)
                              (end-of-buffer)
                              (foolang-indent-all)
                              (foolang-indent-line)
                              (buffer-substring-no-properties (point-min) (point-max)))))
        (end-of-buffer)
        (if (string= target result)
            (progn (insert "ok!")
                   (message "test %s ok" name))
          (push name foolang--test-failures)
          (insert "FAILED!\n")
          (insert "WANTED:\n")
          ;; replace newlines to make trailing whitespace easy to spot
          (insert (replace-regexp-in-string "\n" "|\n" target))
          (insert "\nGOT:\n")
          (insert (replace-regexp-in-string "\n" "|\n" result))
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

(cl-defmacro def-foolang-indent-test (name source target)
  `(let* ((name ,name)
         (test (list ,source ,target))
         (old (assoc-string name foolang--indentation-tests)))
     (if old
         (setcdr old test)
       (push (cons name test) foolang--indentation-tests))
     (when foolang--debug-mode
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

(def-foolang-indent-test "class-indent-7.1"
  "
class Foo { a }
is Bar"
  "
class Foo { a }
    is Bar")

(def-foolang-indent-test "class-indent-8"
  "
class Foo { a }
-- comment
method bar"
  "
class Foo { a }
    -- comment
    method bar")

(def-foolang-indent-test "class-indent-9"
  "
class MethodArgTypeError {}"
  "
class MethodArgTypeError {}")

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
direct method bar"
  "
extend Foo
    direct method bar")

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
class Foo { a }
method bar: x quux: y
x + y"
  "
class Foo { a }
    method bar: x quux: y
        x + y")

(def-foolang-indent-test "method-indent-5"
  "
class Foo { a }
method prefix-
-(self value)"
  "
class Foo { a }
    method prefix-
        -(self value)")

(def-foolang-indent-test "method-indent-6"
  "
class Main {}
direct method + x::Type
x"
  "
class Main {}
    direct method + x::Type
        x")

(def-foolang-indent-test "method-indent-7"
  "
class MyClass { block }
method do: methods
methods do: block.
self"
  "
class MyClass { block }
    method do: methods
        methods do: block.
        self")

(def-foolang-indent-test "body-indent-1"
  "
class Foo { a }
method bar
quux
zot"
  "
class Foo { a }
    method bar
        quux
            zot")

(def-foolang-indent-test "body-indent-2"
  "
class Foo { a }
method bar
quux
zot.
fii"
  "
class Foo { a }
    method bar
        quux
            zot.
        fii")

(def-foolang-indent-test "body-indent-3"
  "
class Foo { a }
method bar
zot { dint.
flint"
  "
class Foo { a }
    method bar
        zot { dint.
              flint")

(def-foolang-indent-test "body-indent-4"
  "
class Foo { a }
method bar
zotarionz bee: x foo: y neg
faa: z neg neg
aas: s"
  "
class Foo { a }
    method bar
        zotarionz bee: x foo: y neg
                  faa: z neg neg
                  aas: s")

(def-foolang-indent-test "body-indent-5"
  "
class Foo { a }
method bar
things
do: { |thing|
thing"
  "
class Foo { a }
    method bar
        things
            do: { |thing|
                  thing")

(def-foolang-indent-test "body-indent-6"
  "
class Foo { a }
method bar
things
; doStuff: x
; moreStuff: y"
  "
class Foo { a }
    method bar
        things
            ; doStuff: x
            ; moreStuff: y")

(def-foolang-indent-test "body-indent-7"
  "
class Foo { a }
method bar
P new: p x: {
let n = p ifTrue: { x } ifFalse: { y }.
Ouch of: n"
  "
class Foo { a }
    method bar
        P new: p x: {
            let n = p ifTrue: { x } ifFalse: { y }.
            Ouch of: n")

(def-foolang-indent-test "body-indent-8"
  "
class Foo { a }
method bar
-- XXX
let foo = Quux x: 1"
  "
class Foo { a }
    method bar
        -- XXX
        let foo = Quux x: 1")

(def-foolang-indent-test "body-indent-9"
  "
class Foo { a }
direct method run: command in: system
-- XXX: decide full/short based on command-line
let benchmarks = Benchmarks output: system output
clock: system clock
full: False.
benchmarks run"
  "
class Foo { a }
    direct method run: command in: system
        -- XXX: decide full/short based on command-line
        let benchmarks = Benchmarks output: system output
                                    clock: system clock
                                    full: False.
        benchmarks run")

(def-foolang-indent-test "body-indent-10"
  "
class Foo { a }
direct method new: system
let compiler = Compiler new.
compiler define: \"system\" as: system.
self _input: system input
_output: system output
_compiler: compiler
_atEof: False
_value: False"
        "
class Foo { a }
    direct method new: system
        let compiler = Compiler new.
        compiler define: \"system\" as: system.
        self _input: system input
             _output: system output
             _compiler: compiler
             _atEof: False
             _value: False")

(def-foolang-indent-test "body-indent-11"
  "
class Foo { a }
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
class Foo { a }
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
class Foo { a }
method readEvalPrint
{
-- Cascade just for fun.
self; prompt; read; eval; print
}
onPanic: { |error context|
_output println: \"ERROR: {error}\".
_output println: context }"
  "
class Foo { a }
    method readEvalPrint
        {
            -- Cascade just for fun.
            self; prompt; read; eval; print
        }
            onPanic: { |error context|
                       _output println: \"ERROR: {error}\".
                       _output println: context }")

(def-foolang-indent-test "body-indent-13"
  "
class Foo { a }
method testPrefix
assertForAll: (1 to: 10)
that: { |n|
let b = Box value: n.
-n == -b }
testing: \"custom prefix method\""
  "
class Foo { a }
    method testPrefix
        assertForAll: (1 to: 10)
        that: { |n|
                let b = Box value: n.
                -n == -b }
        testing: \"custom prefix method\"")

(def-foolang-indent-test "body-indent-14"
  "
class Foo { a }
method check: cond on: x onSuccess: success onFailure: failure
let res = { cond value: x }
onPanic: { |e ctx|
system output println: \"ERROR: {e}\".
failure value }.
res
ifTrue: success
ifFalse: failure"
  "
class Foo { a }
    method check: cond on: x onSuccess: success onFailure: failure
        let res = { cond value: x }
                      onPanic: { |e ctx|
                                 system output println: \"ERROR: {e}\".
                                 failure value }.
        res
            ifTrue: success
            ifFalse: failure")

(def-foolang-indent-test "body-indent-15"
  "
class Foo { a }
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
class Foo { a }
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
class Foo { a }
method testFilePath
let pathX = files path: \"X\".
assert raises: { files path: \"..\" }
error: \"Cannot extend #<FilePath (root)> with ..\"
testing:"
  "
class Foo { a }
    method testFilePath
        let pathX = files path: \"X\".
        assert raises: { files path: \"..\" }
               error: \"Cannot extend #<FilePath (root)> with ..\"
               testing:")

(def-foolang-indent-test "body-indent-17"
  "
class Foo { a }
method foo
bar.

quux."
  "
class Foo { a }
    method foo
        bar.

        quux.")

(def-foolang-indent-test "body-indent-18"
  "
class Foo { a }
method foo
x run: { let x = y bar
quux."
  "
class Foo { a }
    method foo
        x run: { let x = y bar
                             quux.")

(def-foolang-indent-test "body-indent-19"
  "
class Foo { a }
method foo
x run: { bing boing.
let x = y bar
quux."
  "
class Foo { a }
    method foo
        x run: { bing boing.
                 let x = y bar
                             quux.")

(def-foolang-indent-test "body-indent-20"
  "
class Foo { a }
method testRecord
assert true: { let r = {x: -10, y: 52}.
r x + r y == 42 }
testing: \"record creation and accessors\""
  "
class Foo { a }
    method testRecord
        assert true: { let r = {x: -10, y: 52}.
                       r x + r y == 42 }
               testing: \"record creation and accessors\"")

(def-foolang-indent-test "body-indent-21"
  "
class Foo { a }
method foo
"
  "
class Foo { a }
    method foo
        ")

(def-foolang-indent-test "body-indent-22"
  "
class Foo {}
method ^ x
oops
end"
  "
class Foo {}
    method ^ x
        oops
end")

(def-foolang-indent-test "body-indent-23"
  "
class Foo { a }
method foo
defaultNumericPrecision: precision sqrt
-- ie. epsilon
smallNumber: (self computeSmallNumber)"
  "
class Foo { a }
    method foo
        defaultNumericPrecision: precision sqrt
        -- ie. epsilon
        smallNumber: (self computeSmallNumber)")

(def-foolang-indent-test "body-indent-24"
  "
class Foo {}
method addPolynomial: left
Polynomial
coefficients: (self coefficients
with: left coefficients
default: 0.0
collect: { |c1 c2| c1 + c2 } )"
  "
class Foo {}
    method addPolynomial: left
        Polynomial
            coefficients: (self coefficients
                               with: left coefficients
                               default: 0.0
                               collect: { |c1 c2| c1 + c2 } )")

(def-foolang-indent-test "body-indent-24.1"
  "
class Foo {}
method addPolynomial: left
Polynomial
coefficients: (self coefficients
with: left coefficients

default: 0.0

collect: { |c1 c2| c1 + c2 } )"
  "
class Foo {}
    method addPolynomial: left
        Polynomial
            coefficients: (self coefficients
                               with: left coefficients

                               default: 0.0

                               collect: { |c1 c2| c1 + c2 } )")

(def-foolang-indent-test "body-indent-25"
  "
class Foo {}
method bar
\"
                         class {}
method bar
                                             42!
\""
  "
class Foo {}
    method bar
        \"
                         class {}
method bar
                                             42!
\"")

(def-foolang-indent-test "body-indent-26"
  "
class Foo {}
method bar
---
                         class {}
method bar
                                             42!
---"
  "
class Foo {}
    method bar
        ---
                         class {}
method bar
                                             42!
        ---")

(def-foolang-indent-test "body-indent-27"
  "
class Foo {}
method testDirectMethod
self load: \"class ClassDirectMethod \{}
                        direct method gimme1
                            self new gimme2!
                        method gimme2
                            42!
                    end\"
eval: \"ClassDirectMethod gimme1\"
expect: 42!"
  "
class Foo {}
    method testDirectMethod
        self load: \"class ClassDirectMethod \{}
                        direct method gimme1
                            self new gimme2!
                        method gimme2
                            42!
                    end\"
            eval: \"ClassDirectMethod gimme1\"
            expect: 42!")

(def-foolang-indent-test "list-indent-1.1"
  "
class Foo {}
method bar
{
quux
}"
  "
class Foo {}
    method bar
        {
            quux
        }")

(def-foolang-indent-test "list-indent-1.2"
  "
class Foo {}
method bar
Baz do:
{
quux
}"
  "
class Foo {}
    method bar
        Baz do:
        {
            quux
        }")

(def-foolang-indent-test "list-indent-1.3"
  "
class Foo {}
method bar
foo do: {
quux
}"
  "
class Foo {}
    method bar
        foo do: {
            quux
        }")

(def-foolang-indent-test "list-indent-1.4"
  "
class Foo {}
method bar
{ quux.
  quux }"
  "
class Foo {}
    method bar
        { quux.
          quux }")

(def-foolang-indent-test "list-indent-1.5"
  "
class Foo {}
method bar
Baz do: { quux.
  quux }"
  "
class Foo {}
    method bar
        Baz do: { quux.
                  quux }")

(def-foolang-indent-test "list-indent-2"
  "
class Foo {}
method foo
x
ifTrue: { { |i|
block value: default value (array at: i) } }
ifFalse: {}.
42.
m bar: quux
ifTrue: { 42 }.
13
end"
  "
class Foo {}
    method foo
        x
            ifTrue: { { |i|
                        block value: default value (array at: i) } }
            ifFalse: {}.
        42.
        m bar: quux
          ifTrue: { 42 }.
        13
end")

(def-foolang-indent-test "list-indent-3.1"
  "
class Foo {}
method foo
(
    beep)"
"
class Foo {}
    method foo
        (
            beep)")

;; FIXME: 3.2 and 3.3 aren't quite consistent!
(def-foolang-indent-test "list-indent-3.2"
  "
class Foo {}
method foo
({ { x / 0 } on: Error do: { |ex| 42 } }
onPanic: { |ex _| ex startsWith: \"UNHANDLED ERROR\" })"
"
class Foo {}
    method foo
        ({ { x / 0 } on: Error do: { |ex| 42 } }
             onPanic: { |ex _| ex startsWith: \"UNHANDLED ERROR\" })")

(def-foolang-indent-test "list-indent-3.3"
  "
class Foo {}
method foo
({ { x / 0 }
on: Error do: { |ex| 42 } }
onPanic: { |ex _| ex startsWith: \"UNHANDLED ERROR\" })"
 "
class Foo {}
    method foo
        ({ { x / 0 }
               on: Error do: { |ex| 42 } }
         onPanic: { |ex _| ex startsWith: \"UNHANDLED ERROR\" })")

(def-foolang-indent-test "bug-1"
  "
interface Object
method == other
(Self includes: other)
ifTrue: { Output debug
println: \"{self} == {other} (type: {Self})\".
self isEquivalent: other }
end"
  "
interface Object
    method == other
        (Self includes: other)
            ifTrue: { Output debug
                          println: \"{self} == {other} (type: {Self})\".
                      self isEquivalent: other }
end")

(def-foolang-indent-test "end-indent-1"
  "
class Foo { a }

method bar
42

end"
  "
class Foo { a }

    method bar
        42

end")

(def-foolang-indent-test "end-indent-1.1"
  "
class Foo { a }

method bar
42.

end"
  "
class Foo { a }

    method bar
        42.

end")

(def-foolang-indent-test "end-indent-2.0"
  "
class Foo { a }
     end"
  "
class Foo { a }
end")

(def-foolang-indent-test "end-indent-2.1"
  "
class Foo { a }
     end

"
  "
class Foo { a }
end

")

(def-foolang-indent-test "define-indent-1"
  "
define foo
42
end"
  "
define foo
    42
end")

(def-foolang-indent-test "define-indent-2"
  "
define $foo
42
end"
  "
define $foo
    42
end")

(defvar foolang--face-tests)
(setq foolang--face-tests nil)

(defvar foolang--face-test-failures)
(setq foolang--face-test-failures nil)

(cl-defmacro def-foolang-face-test (name source &body target)
  `(let* ((name ,name)
         (test (list ,source ',target))
         (old (assoc-string name foolang--face-tests)))
     (if old
         (setcdr old test)
       (push (cons name test) foolang--face-tests))
     (when foolang--debug-mode
       (apply 'foolang--run-face-test name test))))

(cl-defun foolang--face-specs-match (specs content)
  (dolist (spec specs)
    (multiple-value-bind (ok issue) (foolang--face-spec-matches spec content)
      (unless ok
        (return-from foolang--face-specs-match (values ok issue)))))
  (values t nil))

(cl-defun foolang--face-spec-matches (spec content)
  (destructuring-bind (range face) spec
    (destructuring-bind (start end)
        (if (eq '* range)
            (list 0 (length content))
          range)
      (when (eq '* end)
        (setq end (length content)))
      (loop for i from start below end
            do (let ((actual (get-text-property i 'face content)))
                 (unless (eq face actual)
                   (return-from foolang--face-spec-matches
                     (values nil (list i face actual))))))
      (values t nil))))

(defun foolang--run-face-test (name source target)
  (with-current-buffer (get-buffer-create "*foolang-notes*")
    (end-of-buffer)
    (let ((start (point)))
      (insert "\n" name)
      (lexical-let ((result (with-temp-buffer
                              (foolang-mode)
                              (insert source)
                              (font-lock-fontify-buffer)
                              (buffer-substring (point-min) (point-max))
                              ;; (thing-at-point 'buffer)
                              )))
        (end-of-buffer)
        (multiple-value-bind (ok issue) (foolang--face-specs-match target result)
          (if ok
              (progn (insert " ok!")
                     (message "test %s ok" name))
            (push name foolang--test-failures)
            (let ((oops (format " FAILED! wanted %s at %s, got %s"
                                (second issue) (first issue) (third issue))))
              (insert oops "\n")
              (message "test %s%s" name oops))))))))

(def-foolang-face-test "comment-face-1"
  "-- foo"
  (* font-lock-comment-face))

(def-foolang-face-test "comment-face-2"
  "--123"
  (* font-lock-comment-face))

(def-foolang-face-test "keyword-face-1"
  "foo:bar"
  ((0 3) font-lock-function-name-face)
  ((4 6) nil))

(def-foolang-face-test "method-name-1"
  "method isEmpty"
  ((0 6) font-lock-keyword-face)
  ((7 14) font-lock-function-name-face))

(def-foolang-face-test "method-name-1.2"
  "method second"
  ((0 6) font-lock-keyword-face)
  ((7 13) font-lock-function-name-face))

(def-foolang-face-test "method-name-2"
  "method + other"
  ((0 6) font-lock-keyword-face)
  ((7 8) font-lock-function-name-face)
  ((9 12) nil))

(when (fboundp 'highlight-numbers-mode)
  (def-foolang-face-test "number-face-1"
    "123"
    (* highlight-numbers-number))

  (def-foolang-face-test "number-face-2"
    "123.123"
    ((0 2) highlight-numbers-number)
    ((4 6) highlight-numbers-number))

  (def-foolang-face-test "number-face-3"
    " -123"
    ((2 4) highlight-numbers-number))

  (def-foolang-face-test "number-face-4"
    "-123"
    ((1 3) highlight-numbers-number))

  (def-foolang-face-test "number-face-5"
    "asd-123"
    ((4 6) highlight-numbers-number)))

(defvar foolang--motion-tests)
(setq foolang--motion-tests nil)

(cl-defmacro def-foolang-motion-test (name
                                      source start-position
                                      command
                                      target end-position)
  `(let* ((name ,name)
          (test (list ,source ',start-position ',command ,target ',end-position))
          (old (assoc-string name foolang--motion-tests)))
     (if old
         (setcdr old test)
       (push (cons name test) foolang--motion-tests))
     (when foolang--debug-mode
       (apply 'foolang--run-motion-test name test))))

(defun foolang--run-motion-test (name
                                 source start-position
                                 command
                                 target end-position)
  (with-current-buffer (get-buffer-create "*foolang-notes*")
    (end-of-buffer)
    (let ((start (point)))
      (insert "\n" name)
      (destructuring-bind (result location)
          (with-temp-buffer
            (insert source)
            (foolang-mode)
            (setq indent-tabs-mode nil)
            (destructuring-bind (line col) start-position
              (goto-line line)
              (move-to-column col))
            (apply (first command) (rest command))
            (list (buffer-substring-no-properties
                   (point-min) (point-max))
                  (list (line-number-at-pos) (current-column))))
        (end-of-buffer)
        (if (and (equal result target) (equal location end-position))
            (progn (insert " ok!")
                   (message "test %s ok" name))
          (push name foolang--test-failures)
          (insert " FAILED:\n")
          (unless (equal result target)
            (insert "Bad result, wanted:\n" target "\n")
            (insert "Got:\n" result "\n"))
          (unless (equal location end-position)
            (insert (format "Bad location, wanted %s, got %s\n"
                            end-position location)))
          (message "test %s FAILED!" name))))))

(def-foolang-motion-test "foolang-indent-line-0"
  "
class Foo {}
          method bar"
  (3 1)
  (foolang-indent-line)
  "
class Foo {}
    method bar"
  (3 4))

(def-foolang-motion-test "foolang-indent-line-1"
  "
class Foo {}
 method bar         "
  (3 15)
  (foolang-indent-line)
  "
class Foo {}
    method bar"
  (3 14))

(defun foolang--run-tests ()
  ;; For now these are all indentation tests. When others are
  ;; added this should be split into parts.
  (with-current-buffer (get-buffer-create "*foolang-notes*")
    (erase-buffer))
  (setq foolang--test-failures nil)
  (dolist (test (reverse foolang--indentation-tests))
    (apply 'foolang--run-indentation-test test))
  (dolist (test (reverse foolang--face-tests))
    (apply 'foolang--run-face-test test))
  (dolist (test (reverse foolang--motion-tests))
    (apply 'foolang--run-motion-test test))
  (with-current-buffer "*foolang-notes*"
    (cond (foolang--test-failures
           (display-buffer "*foolang-notes*")
           (user-error "Foolang-mode tests failed!"))
          (t
           (kill-buffer)
           (message "Foolang-mode tests ok!")))))

(progn
  (advice-add 'push-mark :around 'foolang--push-mark-around)
  (unwind-protect
      (foolang--run-tests)
    (advice-remove 'push-mark 'foolang--push-mark-around)))

(provide 'foolang)
