#include <stdio.h>
#include <assert.h>
#include "spine.h"

int main() {
    const char* src =
        "server\n"
        "| host = localhost\n"
        "| port = 8080\n";

    SpineDoc* doc = spine_parse(src);
    assert(doc != NULL);

    if (spine_has_errors(doc)) {
        char* errors = spine_get_errors(doc);
        printf("errors:\n%s\n", errors);
        spine_free_string(errors);
        spine_free_doc(doc);
        return 1;
    }

    const SpineValue* root = spine_doc_root(doc);
    assert(root != NULL);
    assert(spine_value_type(root) == 5);

    const SpineValue* server = spine_object_get(root, "server");
    assert(server != NULL);
    assert(spine_value_type(server) == 5);

    const SpineValue* host = spine_object_get(server, "host");
    assert(host != NULL);
    assert(spine_value_type(host) == 3);

    char* host_str = spine_value_string(host);
    printf("server.host = %s\n", host_str);
    spine_free_string(host_str);

    const SpineValue* port = spine_object_get(server, "port");
    assert(port != NULL);
    assert(spine_value_type(port) == 2);
    printf("server.port = %g\n", spine_value_number(port));

    spine_free_value((SpineValue*)root);
    spine_free_value((SpineValue*)server);
    spine_free_value((SpineValue*)host);
    spine_free_value((SpineValue*)port);
    spine_free_doc(doc);

    printf("all assertions passed\n");
    return 0;
}
