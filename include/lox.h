#ifndef LOX_H
#define LOX_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>

typedef enum {
    VAL_NIL,
    VAL_BOOL,
    VAL_NUMBER,
    VAL_STRING,
    VAL_ARRAY,
    VAL_BUFFER
} LoxValueType;

struct LoxFfiArray;
struct LoxFfiBuffer;

typedef struct {
    LoxValueType type;
    union {
        bool boolean;
        double number;
        const char* string;
        struct LoxFfiArray* array;
        struct LoxFfiBuffer* buffer;
    } as;
} LoxFfiValue;

typedef struct LoxFfiArray {
    LoxFfiValue* elements;
    int length;
    int capacity;
} LoxFfiArray;

typedef struct LoxFfiBuffer {
    unsigned char* bytes;
    int size;
    int capacity;
} LoxFfiBuffer;

typedef LoxFfiValue (*LoxNativeFn)(int arg_count, const LoxFfiValue* args);

typedef struct {
    void (*define_function)(const char* name, int arity, LoxNativeFn fn);
    void (*define_global)(const char* name, LoxFfiValue value);
    LoxFfiValue (*make_nil)();
    LoxFfiValue (*make_bool)(bool b);
    LoxFfiValue (*make_number)(double d);
    LoxFfiValue (*make_string)(const char* s);
    LoxFfiValue (*make_array)(int length, const LoxFfiValue* elements);
    LoxFfiValue (*make_buffer)(int size, const unsigned char* bytes);
    void (*set_error)(const char* message);
    void (*define_function_with_help)(const char* name, int arity, LoxNativeFn fn, const char* help);
} LoxFfiApi;

// Static inline helpers for convenience and readability in modules
static inline LoxFfiValue lox_make_nil() {
    LoxFfiValue val;
    val.type = VAL_NIL;
    val.as.number = 0.0;
    return val;
}

static inline LoxFfiValue lox_make_bool(bool b) {
    LoxFfiValue val;
    val.type = VAL_BOOL;
    val.as.boolean = b;
    return val;
}

static inline LoxFfiValue lox_make_number(double d) {
    LoxFfiValue val;
    val.type = VAL_NUMBER;
    val.as.number = d;
    return val;
}

static inline LoxFfiValue lox_make_string(const char* s) {
    LoxFfiValue val;
    val.type = VAL_STRING;
    val.as.string = s;
    return val;
}

#ifdef __cplusplus
}
#endif

#endif // LOX_H
