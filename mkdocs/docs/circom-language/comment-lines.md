# Comment Lines

In circom, you can place comments in your source code. These comment lines will be ignored by the compiler.  Comments help programmer reading your source code to better understand it. Adding comments to your code is a highly recommended practice.

The comment lines allowed in circom 2.0 are similar to other programming languages like C or C++.

You can write comments on a single line by using `//`:

```text
//Using this, we can comment a line.
```

You can also write a comment at the end of a code line using `//`:

```text
template example(){
    signal input in;   //This is an input signal.
    signal output out; //This is an output signal.
}
```

Finally, you can write comments that span multiple lines using `/*` and `*/`:

```text
/*
All these lines will be 
ignored by the compiler.
*/
```



