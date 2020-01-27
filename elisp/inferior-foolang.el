;; Rudimentary Emacs support, mostly for interfacing with org-mode

;;; Serialization of elisp values to foolang
(defun foolang-print (x &optional nested)
  (cond ((numberp x)
         x)
        ((stringp x)
         (format "'%s'" x))
        ((consp x)
         (format "%s[%s]"
                 (if nested "" "#")
                 (apply 'concat (mapcar (lambda (x)
                                   (format "%s," (foolang-print x t)))
                                        x))))
        ((null x)
         "False")
        ((eq t x)
         "True")
        (t
         (error "Don't know how to serialize for foolang: %s" x))))

(defun foolang-read (x)
  (read x))

;;; Name of the foolang executable.
(defvar foolang-executable
  (if (eq 'windows-nt system-type)
      "foolang.exe"
    "foolang"))

(provide 'foolang)
