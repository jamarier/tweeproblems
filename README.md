# Tweeproblems

## Introduction

Tweeproblems is a transpiler to convert the description of an exercise (a
compact yaml file) into a interactive story to process with twine. The result
is an interactive solve of the problem. Each step shows the context, i.e. what
the student did before, and what are the possibilites (some of them wrong, and
some correct) at this step. After each choice, it is possible to give feedback 
about the choice made.

### Features:

Thanks to SugarBox:

* All niceties from twine (SugarBox v2). It has a elegant style, a menu to save
  and restart the state of the problem...

* Tweeproblems is build above SugarBox without hide this. So it is possible to use
  all SugarBox have to offer: images, sound, pluggins, javascript code, ...

Specific from Tweeproblems:

* Includes MathJax javascript library. So the math expressions are nicely
  rendered on screen.

* Includes an evaluator of math expressions. The writer of exercises says how
  the values are calculated, but it is the program that actually does the
  calculations. This evaluator helps the redaction of the exercise because
  avoid mistyping and rounding issues with raw numbers. (currently only scalar
  magnitudes)

* The evaluator includes an assisted unit check. It's an unsound unit system,
  but avoid add magnitudes of different units or passing parameter of wrong
  units into macros.

* Easy to build macros for the evaluator. (currently the macros are hardcoded
  in source, but add a new macro only require 2 strings: name of macro and
  operation, see section [Adding Macros](#adding-macros)).

* Parallel and concurrent stages: 
  * It is possible to offer alternative paths (the student choose one or another, but
    not both),  
  * Or choose the order of non-dependent tasks (at the end, the student have to
    choose all task marked as concurrents to continue).

## YAML format

(it is recomended to study, yaml format and examples in source dir)

Some previous definitions:

* Each option (a good one or a bad one) is called "gate". The gate has three
  elements:
  * _text_: Text shown at the moment of make a decision.
  * _follow_: Text shown as context after the decision was made. (The context is
    the text in _text_ and in _follow_).
  * _note_: Text shown in a temporal window after the decision was made. It's not
    added to context and it is useful to explain why is a good or bad choice. 

* It is a set of choices (gates). It shows all posibilities and the student
  have to choose one.

* There are three compounds:
  * Sequential: List of passages and compounds. The student have to cope with
    all elems on the list in the same order they are written.

  * Concurrent: List of passages and compounds. The student have to cope with
    all elems on the list in any order. The success of concurrent compound is
    when the student success with each element of it.

  * Alternate: List of passages and compounds. The student have to cope only
    with one of the elems of the list. The success in compound is the success
    in one of the alternatives.

### Gate description 

It's an unique string. A series of markers inform if following text is _text_,
_follow_ or _note_. The markers have to be at the beggining of a row (column 0). 

* "\_\_\_" (three '\_') Following text (in same line too) is _text_.
* "..." (three '.') Following text (in same line too) is _follow_.
* "---" (three '-') Following text (in same line too) is _note_.

It is more confortable to use '|' marker of yaml format to write multiline
text of gates.

The gate is interpreted as a FSM. It starts as _text_, and switch with each
mark. There isn't limit in order or amount or markers.

### Passage description

Hash of hash. Outer have only one key "pass" and inner have three keys:

* _text_: Compulsory. A string. Its the text of the good gate.

* _pre\_bad_: An array/list of strings. Optional. It's a list of wrong gates.
  They are shown at the same time of good gate (defined in _text_). 

  Other way to define it is mistakes you can take instead the good one (defined
  in _text_).

* _post\_bad_: An array/list of string. Optional. It's a list of wrong gates.
  They are shown in the options of the following passage.

  Other way to define it is mistakes you can take after you know the step in
  this passage.

### Compound description

They are a hash of array. The outer key inform about the compound to use: _seq_
for Sequential, _alt_ for alternatives and _con_ for concurrent. In each case,
the array express the posibilities. Array can be from 1 element (without any
sense) or more. 

## Math expressions

As it is said before, tweeproblemas comes with an math evaluator. The math mode
is applied in each group of text surrounded by "{{" and "}}" marks (*on the same line*)

The main structure is:

    "{{"<type of injection> (<variable name> =)? <math expresion> "}}"

* Type of injection gives information of how the math expression is rendered 
  y showed into the story. A math expression can be showed with its definition
  (the mathematical operations to do) or its value (the result of those 
  operations). The type of injection determines which one (or both) are going to
  be shown.

  The type injection is next to "{{" mark without spaces between.

* It is possible assign a name to the result of the expression. And later use 
  that name to retrieve the value of the expression. The name is any valid latex
  eq expression without spaces. So, it is possible to use: "R", "R\_1", "R\_{12}", 
  etc.

  Math expression are not equations. So "x+1=2" doesn't assign 1 to x. It assign the
  value 2 to the variable name "x+1" 

  The binding of the variable name is global. And it keeps for all the rest of the
  interactive story. It is possible, rebind a variable with other expression but, 
  the resulting value has to be the same of the previous expression. So it is easier
  consider the assigns as of constant values (not must be constant expression).

  It is not compulsory assign a name to every expression.

  An example of valid expressions in several passages:

      ... this is some text {{. R_1 = 1 1 +}} more text ...
      ... Another line or passage citing the variable {{, R_1}} ...
      ... this is other line or other passage and {{. R_1 = 2}} ...
      there no problem here. 

* The math expression is the operations and values to be assigned. The expressions 
  are notated as RPN (Reverse Polish Notation) where the arguments are first and 
  the operator is last. 

  In a posterior section [Expressions](#expressions) the ideas after maths expressions
  are developed.

A math expression alone in a row (with other text in same row) will be rendered as 
display mode. In anyother case, the formula will be rendered as inline mode of MathJax

### Type of injection:

The injection defines how to render the expression. Main ideas:
* value is the magnitude: numeric value and units ("no units" is a unit too)

* formula is the calculations to get the value. And it is what it is written 
  in math expression. So, the value of both expression in "{{\_ R = 1 1 +}}" and 
  "{{\; R }}" it is 2, but the formula is "1+1" in first expression and "R" (in 
  second expression).

Types of injection and meaning:
* '.' only show value of Expression (MathJax Format)
* ',' only show formula of Expression (MathJax Format)
* ';' show formula and value separated with '=' sign (MathJax Format)
* '\_' calculate effects of Expression but doesn't show anything. Sometimes
  is more práctical calculate the solution before of data of exercise. With 
  '\_' is possible calculate and bind a variable without show anything.
* '!' calculate value (without units) and insert into text.

### Escaping values

  if tweeproblemas interfere with twine/SugarBox commands, it is possible to 
  inhibit tweeproblemas processing: 
  
  '!' at first column of file copy the rest of line to the output without modification.

  Another possibility is to escape individual characters. There are three escape sequences:
* "\\\\" to escape '\\'
* "\\{" to escape '{'  and 
* "\\}" to escape '}'

  That allow to use directly MathJax notation without launch math evaluator( e.g.
  the expression of MathJax \\( {\\{ 1 + 2 }} \\)  )

## Expressions
  The expressions are notated as RPN. Each element must be separated by spaces. 

  There are four types of elements on expressions:

* Magnitude: numeric value with units.
* Variables: previously defined
* Operators: mathematical operators
* Macros: typical formulas or constants of frequent use.


### MAGNITUDES

* Always without space but '\_' can be used to visually separate parts. 
  All '\_' are omited in parsing.

  Ex: 10\_km, 10km, 10\_000m, 10k\_m, are all the same magnitude

* 2 parts: value and units.

#### Value
value is described as a float number. It can be written using dot or exponent 
format: 1200.0 or 1.2e3

#### Units
unit is a string with arbitrary content. They are not really units. 

But:
* if first letter of all unit is a multiple suffix ('k', 'M', 'm', ...)
      that factor is applied and deleted from unit name. 
* factor only applies if unit string have more than one letter, so "m" is metter not milli.
* factor only applies to the first char of string not in every unit of a compound unit (like "km/h"). 
       
  **BE CAREFUL**: only is the first letter of the string. So km/km is 10^3 of "m/km"

* it's possible to force no factor with '#' factor. So, unit="mother" is 10^-3 of "other"
  and unit="#mother" is 1 of "mother"
    
  character '#' only is needed in magnitude parseing not in expression unit coercion. 

### Variables
Any text not identified as magnitude, operator or formula is marked as Variable. 

In the evaluation of the expression, the value of variable is searched in variable
dictionary. If the variable is not defined. A panic will be thrown.

### Operators

#### Arithmetic operators

At the moment, they are:
"+", "-", "neg" (negate: change of sign), "\*" (product) "/" (division)

#### Expressions units
  Expressions have a faux unit verification. Not always works (sometimes it works, depends on operation). 
  So, there are a especial unit "¿?" meaning unknown unit. Everytime an operation is not cappable to 
  determinate the output unit, the unit change to "¿?". 

  * ":" (expr unit -- expr) unit verification/coercion
    
    if <expr> have a defined unit, this must be <unit> or a panic is thrown

    if <expr> have a unknown unit -> it assign <unit>

    **BE CAREFUL** #' hasn't any special meaning so, "1#mother #mother :" will panic because compares "mother" with "#mother"
    correct way is "1#mother mother :"

  * "::" (expr -- expr) unit verification/coercion
    same as ":" but the unit to use is "without units". Same checks are done

  * Conversion of units
    multiply/divide by conversion factor and assign new unit
    e.g.  "1_#km/h 3.6 / m/s :"

    in case of dirty calculations: change unit without changing the value (multiply by 1)
    e.g.  "value 1 * new_unit :"

#### Asign parameters
  * "!" (expr name -- ) assign expr to name
  * "@" (name -- expr ) retrieve expr saved into name


### Adding Macros

  Las macros no son reentrantes
  Si pueden llamar a otras macros (con cuidadito)

## TODO

* Tips or hints. Give successive tips to solve current steps

* Internalization
  
  Translations of messages into several languages

* Auto builder of passages
  rank all gates by dependencies level (this expression is level 3 because its variables are level 2) 
  and split them into passages automatically

* Random variables and checking of of possibilites

  to generate new problems. 

* Random wrong gates
  The idea is take the formulas used and change the variables for other availables. So generate confuse 
  possibilities
  
* Send information of every step to an server to analyze information

* After the end, give feedback to the user. (you need X steps to finish an exercise of Y steps).  

* Truly unit management
  with suport to product, log, and exponentiation...
