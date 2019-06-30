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

;; This is fairly minimal implementation, with no session support, and
;; which doesn't really play well with toplevel definitions at the
;; moment.

;;; Requirements:

;; foolang.el lives in foolang/elisp in the repository, see
;; https://github.com/nikodemus/foolang

;;; Code:

(require 'ob)
(require 'foolang)

(add-to-list 'org-babel-tangle-lang-exts '("foolang" . "foo"))

(defvar org-babel-default-header-args:foolang '())

(defun org-babel-expand-body:foolang (body params)
  "Expands BODY from org-mode according to PARAMS, arranging for either
   value or printout, and setting up variables."
  (let (vars)
    (dolist (pair params)
      (when (eq :var (car pair))
        (push (cdr pair) vars)))
    (let ((wrapped
           (if vars
               (format "{ %s| %s }%s"
                       (apply 'concat
                              (mapcar (lambda (pair) (format ":%s " (car pair)))
                                      vars))
                       body
                       (apply 'concat
                              (mapcar (lambda (pair) (format " value: %s"
                                                             (foolang-print (cdr pair))))
                                      vars)))
             body)))
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
(defun org-babel-execute:foolang (body params)
  "Execute a block of Foolang code with org-babel.
   This function is called by `org-babel-execute-src-block'"
  (let ((params (org-babel-process-params params)))
    (with-temp-buffer
      (call-process (executable-find foolang-executable)
                    nil t nil "--eval"
                    (org-babel-expand-body:foolang body params))
      (ecase (cdr (assoc :result-type params))
        (value (foolang-read (buffer-string)))
        (output (buffer-string))))))

(provide 'ob-foolang)

