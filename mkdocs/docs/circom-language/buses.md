# Buses
circom 2.2.0 introduces a new feature called __signal buses__. 



## Definition
A bus is a collection of different but related signals grouped under one name. They are similar to structs in programming languages like C++, helping to make the code more organized and easier to manage.

Buses can be defined at the same level as templates and can be used as inputs, intermediates or outputs within a template.

```
bus NameBus(param1,...,paramN){
    //signals, 
    //arrays,
    //other buses...
}

```

In many circuits we have pairs of signals `x` and `y`, which represent the two components of a point. With the new bus feature, we can define a `Point` bus as follows:

```
bus Point(){
    signal x;
    signal y;
}
```

This way, it is clear that `x` and `y` should be understood as a single point rather than two independent signals. 

Using buses, we can modify many templates from the circomlib to make them more readable and organized. Let us consider the `Edwards2Montgomery` template from `montgomery.circom`:

```
template Edwards2Montgomery () {
 input Point() { edwards_point } in ;
 output Point() { montgomery_point } out ;

 out.x <–- (1 + in.y ) / (1 - in.y ) ;
 out.y <–- out.x / in.x ;

 out.x * (1 - in.y ) === (1 + in.y ) ;
 out.y * in.x === out.x ;
 }
```

Here, we have a template with an input `Point` `in` expected to be in Edwards format, and an output `Point` `out` in Montgomery format. 

The power of buses lies in expressing properties about a collection of related signals. For example, the two signals inside the bus `in` (respectively `out`) must satisfy the equations for the Edwards curve (respectively the Montgomery curve). Before circom 2.2.0, this could not be expressed using tags in circom. 
But now, we can tag each bus with the corresponding expected format. 

Besides tagging buses defined in a template, we can also tag their different fields. Let us see this feature in the following example:

```
bus Book () {
    signal {maxvalue} title[50];
    signal {maxvalue} author[50];
    signal {maxvalue} sold_copies;
    signal {maxvalue} year;
};
```

The `Book` bus has four different fields:
signal arrays  `title` and `author` whose letters have a maximum value, the number of sold copies `sold_copies`, and the publication `year`, which also has a maximum value. Using buses makes your code clearer and more readable. It is easier to understand that a `Book` bus represents a book with its fields, rather than dealing with individual signals.

``` 
template BestSeller2024(){
    input Book() book;
    output Book() {best_seller2024} best_book;
    signal check_copies <== LessThan(book.sold_copies.maxvalue)([1000000,book.sold_copies]);
    check_copies === 1;
    signal check_2024 <== IsEqual()([book.year,2024]);
    check_2024 === 1;
    best_book <== book;
}
```

As mentioned above, tags work at both levels: at the level of the whole bus, expressing that the book is a best-seller in 2024 (it sold more than 1 million copies), and at the level of the bus signals, expressing the different correctness properties about the book's fields.

## Approaching a Type System via Buses and Tags
The introduction of buses in circom 2.2.0 brings us closer to having a robust type system. By enforcing compatibility rules in the bus assignments and enabling tagging at both the bus and signal level, buses provide a structured way to manage and verify the relationships between different signals. The combined use of buses and tags emulates the advantages of a traditional type system within circom, enhancing code clarity, reducing errors, and improving overall organization.

When assigning one bus to another, they both need to be the same type of bus. Otherwise, the compiler reports an error.

```
bus B1(){
    signal x;
}

bus B2() {
    signal x;
}

template B1toB2(){
    input B1() b1;
    output B2() b2;
    b2 <== b1;
}

```
For the previous example, the compiler reports:
```
error[T2059]: Typing error found
   ┌─ "example.circom":80:5
   │
   │     b2 <== b1;
   │     ^^^^^^^^^ Assignee and assigned types do not match.
```

In this case, the transformation from one type to another should be explicitly done as follows: `b2.x <== b1.x;`.

Consider again the `BestSeller2024` template and a possible instantiation: `Book seller <== BestSeller2024()(b);` Similar to tags, whenever a template is instantiated, the compiler checks if the type of `b` is equals to `Book`. If it is not, an error is reported. The compiler also checks if the bus' fields have the same tags.

## Buses inside Buses
We can have buses inside the definition other buses, as long as we do not define buses  recursively. To illustrate this, let us consider now, a new kind of bus, `Person`, which contains some information about a person:

```
bus Film() {
    signal title[50];
    signal director[50];
    signal year;
}

bus Date() {
    signal day;
    signal month;
    signal year;
}

bus Person() {
    signal name[50];
    Film() films[10];
    Date() birthday;
}
```

## Parameterized Buses
Buses can have parameters as well. These parameters must be known during compilation  time and can be used to define arrays or other buses inside themselves. 

Let us generalize the `Point` bus for a given dimension. 

```
bus PointN(dim){
    signal x[dim];
}
```

Thanks to this definition, we can define other like lines and figures.  

```
bus Line(dim){
    PointN(dim) start;
    PointN(dim) end;
}

bus Figure(num_sides, dim){
    Line(dim) side[num_sides];
}
```
Notice that the `Figure` bus is defined by two parameters: the number of sides and the dimension of its points. Using this bus, we can define every kind of figure in a very simple way. For instance:

```
bus Triangle2D(){
    Figure(3,2) {well_defined} triangle;
}

bus Square3D(){
    Figure(4,3) {well_defined} square;
}
```

We define a `Triangle2D` bus with three lines whose points are 2-dimensional, and a `Square3D` bus, whose points are 3-dimensional.

```
template well_defined_figure(num_sides, dimension){
    input Figure(num_sides,dimension) t;
    output Figure(num_sides,dimension) {well_defined} correct_t;
    var all_equals = 0;
    var isequal = 0;
    for(var i = 0; i < num_sides; i=i+1){
        for(var j = 0; j < dimension; j=j+1){
            isequal = IsEqual()([t.side[i].end.x[j],t.side[(i+1)%num_sides].start.x[j]]);
            all_equals += isequal;
        }
    }
    all_equals === num_sides;
    correct_t <== t;
}
```

The previous template defines a correctness check for any figure: the ending point of a line must be the starting point of the next line. Otherwise, the figure is not well defined, and the witness generation will fail. 

## Buses as Circuit Inputs
Similar to signals, buses can be part of the main circuit's inputs. Therefore, we must specify their values to generate a witness for the circuit. For each circuit input bus, values can be specified in two ways:

- __Serialized Format__: Indicate the value of every signal, bus, or array field in a single array, following the bus's definition order.
- __JSON Format__: Provide values using a fully qualified JSON format with field names. Note that you cannot mix both methods within a single bus. If you start defining an input using field names, you must use this method consistently throughout.
  
Let us consider again the `Person` bus:
```
bus Film() {
    signal title[2];
    signal director[2];
    signal year;
}

bus Date() {
    signal day;
    signal month;
    signal year;
}

bus Person() {
    signal name[2];
    Film() films[2];
    Date() birthday;
}
```

To indicate values for an input `p` of this kind, we would indicate its values as one of the following ways:

- __Serialized format__:
```
{"p": ["80","82","20","21","30","31","1953","40","41","50","51","1990","1","1","1992"]
}
```
 
 - __JSON format__:
```
{"p": {"name": ["80","82"],
       "films": [
            {   "title": ["20","21"],
                "director": ["30","31"],
                "year": "1953"
            },
            {   "title": ["40","41"],
                "director": ["50","51"],
                "year": "1990"
            }
        ],
       "birthday":
            {   "day": "1",
                "month": "1",
                "year": "1992"
            }
    }
}

```

Like public input signals, public input buses cannot be tagged. Otherwise, the compiler will report an error. 
