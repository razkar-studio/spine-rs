#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

/// Opaque types that C sees as pointers
struct SpineDoc;

struct SpineValue;

extern "C" {

/// # Safety
///
/// `input` must be a valid null-terminated C string.
SpineDoc *spine_parse(const char *input);

/// # Safety
///
/// `doc` must be a valid pointer to a `SpineDoc` or null.
bool spine_has_errors(const SpineDoc *doc);

/// # Safety
///
/// `doc` must be a valid pointer to a `SpineDoc` or null.
char *spine_get_errors(const SpineDoc *doc);

/// # Safety
///
/// `doc` must be a valid pointer to a `SpineDoc` or null.
const SpineValue *spine_doc_root(const SpineDoc *doc);

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
int spine_value_type(const SpineValue *val);

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
bool spine_value_bool(const SpineValue *val);

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
double spine_value_number(const SpineValue *val);

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
char *spine_value_string(const SpineValue *val);

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
char *spine_value_tag(const SpineValue *val);

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
char *spine_value_tag_content(const SpineValue *val);

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
unsigned long spine_array_len(const SpineValue *val);

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
const SpineValue *spine_array_get(const SpineValue *val, unsigned long index);

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
unsigned long spine_object_len(const SpineValue *val);

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
char *spine_object_key(const SpineValue *val, unsigned long index);

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
/// `key` must be a valid null-terminated C string.
const SpineValue *spine_object_get(const SpineValue *val, const char *key);

/// # Safety
///
/// `doc` must be a valid pointer to a `SpineDoc` or null.
void spine_free_doc(SpineDoc *doc);

/// # Safety
///
/// `val` must be a valid pointer to a `SpineValue` or null.
void spine_free_value(SpineValue *val);

/// # Safety
///
/// `s` must be a valid pointer to a C string allocated by Spine, or null.
void spine_free_string(char *s);

}  // extern "C"
