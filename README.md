# pyphen-rs
A pure Rust port of Python's [Pyphen][1] library.

## Usage
```rust
use pyphen_rs;

assert_eq!(pyphen_rs::language_fallback("nl_NL_variant1").unwrap(), "nl_NL");

pyphen_rs::LANGUAGES.with(|l|) {
    assert!(l.borrow().contains_key("nl_NL"));
};

let dic = pyphen_rs::Builder::lang("nl_NL").build().unwrap();
assert_eq!(dic.inserted("lettergrepen"), "let-ter-gre-pen");

let wrap = dic.wrap("autobandventieldopje", 11); // Some(("autoband-", "ventieldopje"))
let iter = dic.iterate("Amsterdam"):
iter.next(); // Some(("Amster", "dam"))
iter.next(); // Some(("Am", "sterdam"))
iter.next(); // None
```

## License

Pyphen-rs is released under the GPL 2.0+/LGPL 2.1+/MPL 1.1 tri-license. See [COPYING.GPL][2], [COPYING.LGPL][3] and [COPYING.MPL][4] for more details.

[1]: https://pyphen.org
[2]: ./COPYING.GPL
[3]: ./COPYING.LGPL
[4]: ./COPYING.MPL