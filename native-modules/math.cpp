#include "lox.h"
#include <cmath>

static LoxFfiValue math_sin(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::sin(args[0].as.number));
}

static LoxFfiValue math_cos(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::cos(args[0].as.number));
}

static LoxFfiValue math_tan(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::tan(args[0].as.number));
}

static LoxFfiValue math_asin(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::asin(args[0].as.number));
}

static LoxFfiValue math_acos(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::acos(args[0].as.number));
}

static LoxFfiValue math_atan(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::atan(args[0].as.number));
}

static LoxFfiValue math_atan2(int argc, const LoxFfiValue* args) {
    if (argc < 2 || args[0].type != VAL_NUMBER || args[1].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::atan2(args[0].as.number, args[1].as.number));
}

static LoxFfiValue math_sinh(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::sinh(args[0].as.number));
}

static LoxFfiValue math_cosh(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::cosh(args[0].as.number));
}

static LoxFfiValue math_tanh(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::tanh(args[0].as.number));
}

static LoxFfiValue math_exp(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::exp(args[0].as.number));
}

static LoxFfiValue math_log(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::log(args[0].as.number));
}

static LoxFfiValue math_log10(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::log10(args[0].as.number));
}

static LoxFfiValue math_log2(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::log2(args[0].as.number));
}

static LoxFfiValue math_pow(int argc, const LoxFfiValue* args) {
    if (argc < 2 || args[0].type != VAL_NUMBER || args[1].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::pow(args[0].as.number, args[1].as.number));
}

static LoxFfiValue math_sqrt(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::sqrt(args[0].as.number));
}

static LoxFfiValue math_ceil(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::ceil(args[0].as.number));
}

static LoxFfiValue math_floor(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::floor(args[0].as.number));
}

static LoxFfiValue math_round(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::round(args[0].as.number));
}

static LoxFfiValue math_abs(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    return lox_make_number(std::abs(args[0].as.number));
}

extern "C" {

#ifdef _WIN32
__declspec(dllexport)
#endif
void lox_module_init(const LoxFfiApi* api) {
    api->define_function("sin", 1, math_sin);
    api->define_function("cos", 1, math_cos);
    api->define_function("tan", 1, math_tan);
    api->define_function("asin", 1, math_asin);
    api->define_function("acos", 1, math_acos);
    api->define_function("atan", 1, math_atan);
    api->define_function("atan2", 2, math_atan2);
    api->define_function("sinh", 1, math_sinh);
    api->define_function("cosh", 1, math_cosh);
    api->define_function("tanh", 1, math_tanh);
    api->define_function("exp", 1, math_exp);
    api->define_function("log", 1, math_log);
    api->define_function("log10", 1, math_log10);
    api->define_function("log2", 1, math_log2);
    api->define_function("pow", 2, math_pow);
    api->define_function("sqrt", 1, math_sqrt);
    api->define_function("ceil", 1, math_ceil);
    api->define_function("floor", 1, math_floor);
    api->define_function("round", 1, math_round);
    api->define_function("abs", 1, math_abs);

    api->define_global("pi", lox_make_number(3.14159265358979323846));
    api->define_global("e", lox_make_number(2.71828182845904523536));
    api->define_global("tau", lox_make_number(6.28318530717958647692));
}

}
