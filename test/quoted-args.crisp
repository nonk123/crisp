(defun unless ['condition 'action]
  (if ,condition
      nil
    ,action))

(let 'output 99)

(unless nil
  (set 'output 100))

(defun do-twice ['action]
  ,action
  ,action)

(do-twice (set 'output (+ output 10)))

(debug output)
