;;; ob-foolang.el --- org-babel functions for foolang evaluation

;; Copyright (2019) Nikodemus Siivola <nikodemus@random-state.net>

;; Author: NIkodemus Siivola
;; Keywords: literate programming, reproducible research
;; Homepage: https://orgmode.org
;; Version: 0.01

;;; License:

;; This program is free software; you can redistribute it and/or modify
;; it under the terms of the GNU General Public License as published by
;; the Free Software Foundation; either version 3, or (at your option)
;; any later version.
;;
;; This program is distributed in the hope that it will be useful,
;; but WITHOUT ANY WARRANTY; without even the implied warranty of
;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;; GNU General Public License for more details.
;;
;; You should have received a copy of the GNU General Public License
;; along with GNU Emacs; see the file COPYING.  If not, write to the
;; Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
;; Boston, MA 02110-1301, USA.

;;; Commentary:

;; If you have questions as to any of the portions of the file defined
;; below please look to existing language support for guidance.
;;
;; If you are planning on adding a language to org-babel we would ask
;; that if possible you fill out the FSF copyright assignment form
;; available at https://orgmode.org/request-assign-future.txt as this
;; will make it possible to include your language support in the core
;; of Org-mode, otherwise unassigned language support files can still
;; be included in the contrib/ directory of the Org-mode repository.

;;; Requirements:

;; Use this section to list the requirements of this language.  Most
;; languages will require that at least the language be installed on
;; the user's system, and the Emacs major mode relevant to the
;; language be installed as well.

;;; Code:
(require 'ob)
(require 'ob-ref)
(require 'ob-comint)
(require 'ob-eval)
;; possibly require modes required for your language

;; optionally define a file extension for this language
(add-to-list 'org-babel-tangle-lang-exts '("foolang" . "foo"))

;; optionally declare default header arguments for this language
(defvar org-babel-default-header-args:foolang '())

;; This function expands the body of a source code block by doing
;; things like prepending argument definitions to the body, it should
;; be called by the `org-babel-execute:foolang' function below.
(defun org-babel-expand-body:foolang (body params)
  "Expand BODY according to PARAMS, return the expanded body."
  (let (vars)
    (dolist (pair params)
      (when (eq :var (car pair))
        (push (cdr pair) vars)))
    (let ((wrapped (if vars
                       (format "{ %s| %s }%s"
                               (apply 'concat
                                      (mapcar (lambda (pair) (format ":%s " (car pair)))
                                                   vars))
                               body
                               (apply 'concat
                                      (mapcar (lambda (pair) (format " value: %s" (cdr pair)))
                                              vars)))
                     body)))
      (message "wrapped: %s" wrapped)
      (ecase (cdr (assoc :result-type params))
        (value (format "System stdout print: (%s) toString" wrapped))
        (output wrapped)))))

;; This is the main function which is called to evaluate a code
;; block.
;;
;; This function will evaluate the body of the source code and
;; return the results as emacs-lisp depending on the value of the
;; :results header argument
;; - output means that the output to STDOUT will be captured and
;;   returned
;; - value means that the value of the last statement in the
;;   source code block will be returned
;;
;; The most common first step in this function is the expansion of the
;; PARAMS argument using `org-babel-process-params'.
;;
;; Please feel free to not implement options which aren't appropriate
;; for your language (e.g. not all languages support interactive
;; "session" evaluation).  Also you are free to define any new header
;; arguments which you feel may be useful -- all header arguments
;; specified by the user will be available in the PARAMS variable.
(defun org-babel-execute:foolang (body params)
  "Execute a block of Foolang code with org-babel.
This function is called by `org-babel-execute-src-block'"
  (with-temp-buffer
    (call-process (executable-find foolang-executable)
                  nil t nil "--eval"
                  (org-babel-expand-body:foolang
                   body (org-babel-process-params params)))
    (buffer-string)))

;; This function should be used to assign any variables in params in
;; the context of the session environment.
(defun org-babel-prep-session:foolang (session params)
  "Prepare SESSION according to the header arguments specified in PARAMS."
  )

(defun org-babel-foolang-var-to-foolang (var)
  "Convert an elisp var into a string of foolang source code
specifying a var of the same value."
  (format "%S" var))

(defun org-babel-foolang-table-or-string (results)
  "If the results look like a table, then convert them into an
Emacs-lisp table, otherwise return the results as a string."
  )

(defun org-babel-foolang-initiate-session (&optional session)
  "If there is not a current inferior-process-buffer in SESSION then create.
Return the initialized session."
  (message "Session: %s" session)
  (unless (string= session "none")
    ))

(provide 'ob-foolang)
;;; ob-foolang.el ends here
