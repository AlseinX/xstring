# XString

`XString` is an immutable owned string (and also bytes), sized 2 pointers, that may conditionally hold the data inlined, statically, or within ref counted heap allocated memory, which makes it cheap to clone and pass around.

It has a certain memory representation and could be easily passed through FFI boundaries.