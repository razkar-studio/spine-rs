# spine-abi

The C ABI for the Spine parser. Compiles to a `.so`/`.dll`/`.dylib` and `.wasm` for use from any language.

> This crate is published to crates.io to reserve the namespace. Most users should download the pre-built `.so`/`.dll`/`.dylib`/`.wasm` artifacts from the GitHub CI Artifacts workflow runs instead of depending on this crate directly.

## Usage from C

```c
#include "spine.h"

SpineDoc* doc = spine_parse("host = localhost\n");
if (spine_has_errors(doc)) {
    char* errors = spine_get_errors(doc);
    fprintf(stderr, "%s\n", errors);
    spine_free_string(errors);
    spine_free_doc(doc);
    return 1;
}

const SpineValue* root = spine_doc_root(doc);
const SpineValue* host = spine_object_get(root, "host");
char* str = spine_value_string(host);
printf("%s\n", str);

spine_free_string(str);
spine_free_value((SpineValue*)host);
spine_free_value((SpineValue*)root);
spine_free_doc(doc);
```

## Memory

Every value and string returned by the API must be freed with the corresponding `spine_free_*` function.

## Contributing

If you have free time and when Spine develops more, you are more than welcome to make a Spine port for any language that doesn't already have an [_official / officially acknowledged port_](../PORTS.md) using this ABI!

## License

This project is licensed under the [BSD 3-Clause License](LICENSE) (or <https://opensource.org/license/bsd-3-clause>).

In short, you’re free to use, modify, and distribute this software however you want, as long as you:

* Keep the original copyright notice and license text.
* Don’t use my name or the contributors’ names to promote forked products without permission.

Cheers, RazkarStudio.

Copyright © 2026 RazkarStudio and Spine Contributors. All rights reserved.
