#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

/// Opaque type representing a parsed Spine document.
///
/// C code receives and returns this as an opaque pointer. Use the
/// accessor functions to inspect the document's root value and errors.
struct SpineDoc;

/// Opaque type representing a Spine value.
///
/// Returned by document and value accessors. Must not be retained
/// after the parent document is freed.
struct SpineValue;

/// Parser and spec metadata exposed by the ABI layer.
///
/// The caller MUST free the strings with `spine_free_format_details`.
struct SpineFormatDetails {
  /// The parser version string.
  const char *version;
  /// The spec version this parser targets.
  const char *spec;
  /// The backend type (`"native"` or `"wasm"`).
  const char *backend;
};

extern "C" {

/// Parses Spine source text and returns a `SpineDoc`.
///
/// The returned document must be freed with `spine_free_doc`.
///
/// # Safety
///
/// `input` must be a valid null-terminated C string.
SpineDoc *spine_parse(const char *input);

/// Parses Spine source with an associated filename for error messages.
///
/// # Safety
///
/// `input` must be a valid null-terminated C string.
/// `filename` must be a valid null-terminated C string, or null.
SpineDoc *spine_parse_named(const char *input, const char *filename);

/// Returns `true` if the document contains parse errors.
///
/// # Safety
///
/// `doc` must be a valid pointer to a `SpineDoc` or null.
bool spine_has_errors(const SpineDoc *doc);

/// Returns a newline-separated string of all parse errors.
///
/// The returned string must be freed with `spine_free_string`.
///
/// # Safety
///
/// `doc` must be a valid pointer to a `SpineDoc` or null.
char *spine_get_errors(const SpineDoc *doc);

/// Returns a pointer to the document's root value, or null for empty documents.
///
/// # Safety
///
/// `doc` must be a valid pointer to a `SpineDoc` or null.
const SpineValue *spine_doc_root(const SpineDoc *doc);

/// Returns the type of a Spine value as an integer:
///
/// - `-1`: null pointer
/// - `0`: Null
/// - `1`: Bool
/// - `2`: Number
/// - `3`: String
/// - `4`: Array
/// - `5`: Object
/// - `6`: Tagged
///
/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
int spine_value_type(const SpineValue *val);

/// Returns the boolean value, or `false` if not a boolean.
///
/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
bool spine_value_bool(const SpineValue *val);

/// Returns the numeric value, or `0.0` if not a number.
///
/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
double spine_value_number(const SpineValue *val);

/// Returns the string value as a C string, or null if not a string.
///
/// The returned string must be freed with `spine_free_string`.
///
/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
char *spine_value_string(const SpineValue *val);

/// Returns the tag name of a tagged value, or null if not tagged.
///
/// The returned string must be freed with `spine_free_string`.
///
/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
char *spine_value_tag(const SpineValue *val);

/// Returns the content of a tagged value, or null if not tagged.
///
/// The returned string must be freed with `spine_free_string`.
///
/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
char *spine_value_tag_content(const SpineValue *val);

/// Returns the number of elements in an array, or `0` if not an array.
///
/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
unsigned long spine_array_len(const SpineValue *val);

/// Returns the element at `index` from an array, or null if out of bounds.
///
/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
const SpineValue *spine_array_get(const SpineValue *val, unsigned long index);

/// Returns the number of fields in an object, or `0` if not an object.
///
/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
unsigned long spine_object_len(const SpineValue *val);

/// Returns the key at `index` from an object's field list, or null if
/// out of bounds. The returned string must be freed with `spine_free_string`.
///
/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
char *spine_object_key(const SpineValue *val, unsigned long index);

/// Returns the value at `key` from an object, or null if not found.
///
/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
/// `key` must be a valid null-terminated C string.
const SpineValue *spine_object_get(const SpineValue *val, const char *key);

/// Frees a `SpineDoc` allocated by the parser.
///
/// # Safety
///
/// `doc` must be a valid pointer to a `SpineDoc` or null.
void spine_free_doc(SpineDoc *doc);

/// Frees a `SpineValue` allocated by the library.
///
/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
void spine_free_value(SpineValue *val);

/// Frees a string allocated by the library.
///
/// # Safety
///
/// `s` must be a valid pointer to a C string allocated by Spine, or null.
void spine_free_string(char *s);

/// Returns the parser version, spec version, and backend type.
///
/// The caller MUST free the returned struct with `spine_free_format_details`.
///
/// # Safety
///
/// Calling this function is always safe; it returns a pointer to a static
/// structure with format details.
SpineFormatDetails spine_format_details();

/// Frees the strings returned by `spine_format_details`.
///
/// # Safety
///
/// `details` must have been returned by `spine_format_details` and not
/// freed already.
void spine_free_format_details(SpineFormatDetails details);

/// Parses Spine source and returns the AST as a JSON string.
///
/// The JSON output includes format metadata and either the parsed
/// value or error information:
///
/// ```json
/// {"version":"0.1.0","spec":"1.0.0","backend":"native","ok":true,"value":{...}}
/// {"version":"0.1.0","spec":"1.0.0","backend":"native","ok":false,"errors":[...]}
/// ```
///
/// The returned string must be freed with `spine_free_string`.
///
/// # Safety
///
/// `input` must be a valid null-terminated C string.
char *spine_parse_json(const char *input);

}  // extern "C"
