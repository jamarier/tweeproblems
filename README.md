## MAGNITUDES

* Always without space but '____' can be used to visually separate parts. 
  All '_' are omited.

  Ex: 10_km, 10km, 10_000m, 10k_m, are all the same

* 2 parts: value and units.
  * value is described as a float number
  * unit is a string with arbitrary content
    * if first letter of all unit is a multiple suffix ('k', 'M', 'm', ...)
      that factor is applies. 
    * factor only applies if unit string have more than one letter, so "m" is metter not milli.
    * factor only applies to the first char of string not in every unit of a compound unit (like "km/h"). 
      _ BE CAREFUL: _ only is the first letter of the string. So km/km is 10^3 of "m/km"
    * it's possible to force no factor with '#' factor. 
      * So unit="mother" is 10^-3 of "other" 
      * and unit="#mother" is 1 of "mother"
    
      # character '#' only is needed in magnitude parseing not in expression unit coercion. 

## Injecting formulas 
  Passages are read line by line

  * '!' at first copy the rest of line to the output without modification

  * In source: formulas and values are inside a double with this structure:
    "{{" <type of injection> (<Variablename> =)? <Expresion> "}}"

  * Type of injection:
    * "." only show value of Expression (MathJax Format)
    * "," only show formula of Expression (MathJax Format)
    * ";" show formula and value (MathJax Format)
    * "_" calculate effects of Expression but doesn't show anything
    * "!" calculate value (without units) and insert into text.

## Expressions
  Expressions are a RPN notation.

### Expressions units
  Expressions have a faux unit verification. Not always works (sometimes it works, depends on operation). 
  So, there are a especial unit "¿?" meaning unknown unit. Everytime an operation is not cappable to 
  determinate output unit, the unit change to "¿?". 

  * ":" (expr unit -- expr) unit verification/coercion
    
    if <expr> have a definite unit, this must be <unit> or a panic is thrown

    if <expr> have a unknown unit -> it assign <unit>

    _BE CAREFUL_ #' hasn't any special meaning so, "1#mother #mother :" will panic because compares "mother" with "#mother"
    correct way is "1#mother mother :"

  * "::" (expr -- expr) unit verification/coercion
    same as ":" but the unit to use is "without units". Same checks are done

  * Conversion of units
    multiply/divide by conversion factor and assign new unit
    e.g.  "1_#km/h 3.6 / m/s :"

    in case of dirty calculations: change unit without changing the value (multiply by 1)
    e.g.  "value 1 * new_unit :"


## TODO
* Detect Passage without any good answer

* Internalization
  
  Translations of messages into several languages

* Detector of paragraph formulas

  If a formula is the only text in the line, put in display mode not inline mode

* Transicion gates

  Possibility of add comments after selecting a gate (e.g. why is incorrect)

* Parallel Passages: 

  generate two or more independ gates to achieve the same step
  
* Auto builder of passages
  rank all gates by dependencies level (this expression is level 3 because its variables are level 2) 
  and split them into passages automatically

* Random variables and checking of of possibilites

  to generate new problems. 

* Random wrong gates
  The idea is take the formulas used and change the variables for other availables. So generate confuse 
  possibilities
  
* Send information of every step to an server to analyze information

* Truly unit management
  with suport to product, log, and exponentiation...
