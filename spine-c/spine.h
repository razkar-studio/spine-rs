#ifndef SPINE_H
#define SPINE_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct SpineDoc SpineDoc;
typedef struct SpineValue SpineValue;

SpineDoc* spine_parse(const char* input);

bool spine_has_errors(const SpineDoc* doc);
char* spine_get_errors(const SpineDoc* doc);

const SpineValue* spine_doc_root(const SpineDoc* doc);

int spine_value_type(const SpineValue* val);

bool spine_value_bool(const SpineValue* val);
double spine_value_number(const SpineValue* val);
char* spine_value_string(const SpineValue* val);

char* spine_value_tag(const SpineValue* val);
char* spine_value_tag_content(const SpineValue* val);

unsigned long spine_array_len(const SpineValue* val);
const SpineValue* spine_array_get(const SpineValue* val, unsigned long index);

unsigned long spine_object_len(const SpineValue* val);
char* spine_object_key(const SpineValue* val, unsigned long index);
const SpineValue* spine_object_get(const SpineValue* val, const char* key);

void spine_free_doc(SpineDoc* doc);
void spine_free_value(SpineValue* val);
void spine_free_string(char* s);

#ifdef __cplusplus
}
#endif

#endif
