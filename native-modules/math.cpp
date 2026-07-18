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
    api->define_function_with_help("sin", 1, math_sin, "sin(x)\nCalculates the sine of the angle in radians.\nArguments:\n  x: Number representing the angle in radians.\nReturns: Number.\nError Cases: Returns Nil if argument is not a number.");
    api->define_function_with_help("cos", 1, math_cos, "cos(x)\nCalculates the cosine of the angle in radians.\nArguments:\n  x: Number representing the angle in radians.\nReturns: Number.\nError Cases: Returns Nil if argument is not a number.");
    api->define_function_with_help("tan", 1, math_tan, "tan(x)\nCalculates the tangent of the angle in radians.\nArguments:\n  x: Number representing the angle in radians.\nReturns: Number.\nError Cases: Returns Nil if argument is not a number.");
    api->define_function_with_help("asin", 1, math_asin, "asin(x)\nCalculates the arcsine of x.\nArguments:\n  x: Number in range [-1.0, 1.0].\nReturns: Number representing angle in radians.\nError Cases: Returns Nil if argument is invalid.");
    api->define_function_with_help("acos", 1, math_acos, "acos(x)\nCalculates the arccosine of x.\nArguments:\n  x: Number in range [-1.0, 1.0].\nReturns: Number representing angle in radians.\nError Cases: Returns Nil if argument is invalid.");
    api->define_function_with_help("atan", 1, math_atan, "atan(x)\nCalculates the arctangent of x.\nArguments:\n  x: Number.\nReturns: Number representing angle in radians.\nError Cases: Returns Nil if argument is not a number.");
    api->define_function_with_help("atan2", 2, math_atan2, "atan2(y, x)\nCalculates the arctangent of y/x using sign to determine the quadrant.\nArguments:\n  y: Number representing y coordinate.\n  x: Number representing x coordinate.\nReturns: Number representing angle in radians.\nError Cases: Returns Nil if arguments are not numbers.");
    api->define_function_with_help("sinh", 1, math_sinh, "sinh(x)\nCalculates the hyperbolic sine of x.\nArguments:\n  x: Number.\nReturns: Number.\nError Cases: Returns Nil if argument is not a number.");
    api->define_function_with_help("cosh", 1, math_cosh, "cosh(x)\nCalculates the hyperbolic cosine of x.\nArguments:\n  x: Number.\nReturns: Number.\nError Cases: Returns Nil if argument is not a number.");
    api->define_function_with_help("tanh", 1, math_tanh, "tanh(x)\nCalculates the hyperbolic tangent of x.\nArguments:\n  x: Number.\nReturns: Number.\nError Cases: Returns Nil if argument is not a number.");
    api->define_function_with_help("exp", 1, math_exp, "exp(x)\nCalculates the exponential function of x (e^x).\nArguments:\n  x: Number.\nReturns: Number.\nError Cases: Returns Nil if argument is not a number.");
    api->define_function_with_help("log", 1, math_log, "log(x)\nCalculates the natural logarithm (base e) of x.\nArguments:\n  x: Positive Number.\nReturns: Number.\nError Cases: Returns Nil if argument is invalid.");
    api->define_function_with_help("log10", 1, math_log10, "log10(x)\nCalculates the base-10 logarithm of x.\nArguments:\n  x: Positive Number.\nReturns: Number.\nError Cases: Returns Nil if argument is invalid.");
    api->define_function_with_help("log2", 1, math_log2, "log2(x)\nCalculates the base-2 logarithm of x.\nArguments:\n  x: Positive Number.\nReturns: Number.\nError Cases: Returns Nil if argument is invalid.");
    api->define_function_with_help("pow", 2, math_pow, "pow(base, exp)\nCalculates base raised to the exponent power.\nArguments:\n  base: Number representing the base.\n  exp: Number representing the exponent.\nReturns: Number.\nError Cases: Returns Nil if arguments are not numbers.");
    api->define_function_with_help("sqrt", 1, math_sqrt, "sqrt(x)\nCalculates the square root of x.\nArguments:\n  x: Non-negative Number.\nReturns: Number.\nError Cases: Returns Nil if argument is invalid.");
    api->define_function_with_help("ceil", 1, math_ceil, "ceil(x)\nReturns the smallest integer greater than or equal to x.\nArguments:\n  x: Number.\nReturns: Number representing ceiling value.\nError Cases: Returns Nil if argument is not a number.");
    api->define_function_with_help("floor", 1, math_floor, "floor(x)\nReturns the largest integer less than or equal to x.\nArguments:\n  x: Number.\nReturns: Number representing floor value.\nError Cases: Returns Nil if argument is not a number.");
    api->define_function_with_help("round", 1, math_round, "round(x)\nReturns the value of x rounded to the nearest integer.\nArguments:\n  x: Number.\nReturns: Number representing rounded value.\nError Cases: Returns Nil if argument is not a number.");
    api->define_function_with_help("abs", 1, math_abs, "abs(x)\nReturns the absolute value of x.\nArguments:\n  x: Number.\nReturns: Number representing absolute value.\nError Cases: Returns Nil if argument is not a number.");

    api->define_global("pi", lox_make_number(3.14159265358979323846));
    api->define_global("e", lox_make_number(2.71828182845904523536));
    api->define_global("tau", lox_make_number(6.28318530717958647692));
}

}
