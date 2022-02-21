# Chimera Specification Document

## Program Structure

### Item

A Chimera file contains a set of **items**, currently this is limited to let-defintions.

```
let answerToEverything = 42
```

Let-definitions have **expressions** on their right hand side, this has the effect of both
evaluating the expression _eagerly_ and binding the result to the name on the left hand side.

### Expression

Expressions describe a computation that can return a value of a certain _type_, currently we
have four known kinds of expressions.

#### Name

Any usage of a bound name will return the result of evaluating the expression it refers to.

```
answerToEverything
```

#### Literal

Literals are currently limited to unsigned integers.

#### Lambda

This expressions represents a function, also known as an _abstraction_, it can be defined to
take any number of arguments. A function can be bound to a name or given as an agument to
other functions.

```
let identity = x => x
```

#### Application

Applications involve a **lambda** expression on the left hand side and any number of expression
on the right hand side. This has the effect of binding the arguments in the function body and
returning the result of evaluation.

```
let zero = identity 0
```
