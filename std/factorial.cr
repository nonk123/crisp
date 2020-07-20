(let 'result 1)

(while (/= input 0)
  (set 'result (* result input))
  (set 'input (- input 1)))

(debug result)
result
