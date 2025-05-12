# XString

`XString` is an immutable owned string (and also bytes), sized 2 pointers, that may conditionally hold the data inlined, statically, or within ref counted heap allocated memory, which makes it cheap to clone and pass around.

It has a certain memory representation and could be easily passed through FFI boundaries.

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT Or Apache-2.0 licensed][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/xstring?style=flat-square
[crates-url]: https://crates.io/crates/xstring
[docs-badge]: https://img.shields.io/docsrs/xstring?style=flat-square
[docs-url]: https://docs.rs/xstring/latest/xstring/
[license-badge]: https://img.shields.io/crates/l/xstring?style=flat-square
[license-url]: https://github.com/AlseinX/xstring/blob/master/LICENSE
[actions-badge]: https://img.shields.io/github/actions/workflow/status/AlseinX/xstring/check.yml?style=flat-square
[actions-url]: https://github.com/AlseinX/xstring/actions?query=workflow%3ACI+branch%3Aslaveholder