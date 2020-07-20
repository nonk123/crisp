(let 'result 1)
(let 'input [1 2 3 4 5])

(while input
  (set 'result (* result (car input)))
  (set 'input (cdr input)))

(debug result)
result
