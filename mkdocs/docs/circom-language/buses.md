# Buses
circom 2.2.0 introduces a new feature called __signal buses__. 



## Definition
A bus is a collection of different but related signals grouped under one name. They are similar to structs in programming languages like C++, helping to make the code more organized and easier to manage.

Buses can be defined at the same level as templates and can be used as inputs, intermediates or ouputs within a template.

```
bus NameBus(param1,...,paramN){
    //signals, 
    //arrays,
    //other buses...
}

```

In many circuits we have pairs of signals `x` and `y`, which represent the two components of a point. With the new bus feature, we can define a `Point` bus as follows:

```
bus Point{
    signal x;
    signal y;
}
```

This way, it is clear that `x` and `y` should be understood as a single point rather than two independent signals. 

Using buses, we can modify many templates from the circomlib to make them more readable and organized. Let us consider the `Edwards2Montgomery` template from `montgomery.circom`:

```
template Edwards2Montgomery () {
 Point input { edwards_point } in ;
 Point output { montgomery_point } out ;

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
bus Book {
    signal {maxvalue} title[50];
    signal {maxvalue} author[50];
    signal pages;
    signal {maxvalue} year;
};
```

The `Book` bus has four different fields:
signal arrays  `title` and `author` whose letters have a maximum value, the number of `pages`, and the publication `year`, which also has a maximum value. Using buses makes your code clearer and more readable. It is easier to understand that a `Book` bus represents a book with its fields, rather than dealing with individual signals.

``` 
template OldBook(){
    Book input book;
    Book output {old} old_book;
    signal check <== LessThan(book.year.maxvalue)([book.year,1950]);
    check === 1;
    old_book <== book;
}
```

As mentioned above, tags work at both levels: at the level of the whole bus, expressing that the book was written before 1950, and at the level of the bus signals, expressing the different correctness properties about them.

## Approaching a Type System via Buses and Tags
The introduction of buses in circom 2.2.0 brings us closer to having a robust type system. By enforcing compatibility rules in the bus assignments and enabling tagging at both the bus and signal level, buses provide a structured way to manage and verify the relationships between different signals. The combined use of buses and tags emulates the advantages of a traditional type system within circom, enhancing code clarity, reducing errors, and improving overall organization.

When assigning one bus to another, they both need to be the same type of bus. Otherwise, the compiler reports an error.

```
bus B1(){
    signal x;
}

bus B2{
    signal x;
}

template B1toB2(){
    B1 input b1;
    B2 output b2;
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

Consider again the `OldBook` template and a possible instantiation: `Book old <== OldBook()(b);` Similar to tags, whenever a template is instantiated, the compiler checks if the type of `b` is equals to `Book`. If it is not, an error is reported. The compiler also checks if the bus' fields have the same tags.

## Buses inside Buses
We can have buses inside the definition other buses, as long as we do not define buses  recursively. To illustrate this, let us consider now, a new kind of bus, `Person`, which contains some information about a person:

```
bus Date{
    signal day;
    signal month;
    signal year;
}

bus Person{
    signal name[50];
    Book books[10];
    Date birthday;
}
```

## Parameterized Buses
Buses can have parameters as well. These parameters must be known during compilation  time and can be used to define arrays or other buses inside themselves. 

Let us generalize the `Point` bus for a given dimension. 

```
bus Point(dim){
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
    Figure(3,2) triangle;
}

bus Square3D(){
    Figure(4,3) square;
}
```

We define a `Triangle2D` bus with three lines whose points are 2-dimensional, and a `Square3D` bus, whose points are 3-dimensional.

```
template well_defined_figure(num_sides, dimension){
    Figure(num_sides,dimension) input t;
    Figure(num_sides,dimension) {correct_t} output t;
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
Similar to signals, buses can be part of the inputs of the main circuit. Thus, we must indicate their values if we want to generate a witness for the circuit. As usual, we indicate the value of the input buses following a JSON format. Let us consider again the `Person` bus and an input `p` of this kind, we would indicate its values as follows:

```
["p": {"name": ["80","82",...], 
       "books": [
            {   "title": [...],
                "author": [...],
                "pages": "30",
                "year": "1953"
            },
                ...,
            {   "title": [...],
                "author": [...],
                "pages": "121",
                "year": "1990"
            }
        ], 
       "birthday": 
            {   "day": "1", 
                "month": "1", 
                "year": "1992"
            }
    }
]
```
