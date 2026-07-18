#include "lox.h"
#include <cstdio>
#include <cstring>
#include <map>
#include <string>
#include <vector>
#include <filesystem>

namespace fs = std::filesystem;

static std::map<double, FILE*> open_files;
static double next_handle = 1.0;
static const LoxFfiApi* g_api = nullptr;

static LoxFfiValue io_open(int argc, const LoxFfiValue* args) {
    if (argc < 2 || args[0].type != VAL_STRING || args[1].type != VAL_STRING) return lox_make_nil();
    std::string path = args[0].as.string;
    std::string mode = args[1].as.string;

    FILE* f = std::fopen(path.c_str(), mode.c_str());
    if (!f) return lox_make_nil();

    double handle = next_handle++;
    open_files[handle] = f;
    return lox_make_number(handle);
}

static LoxFfiValue io_close(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_bool(false);
    double handle = args[0].as.number;
    auto it = open_files.find(handle);
    if (it == open_files.end() || !it->second) return lox_make_bool(false);

    std::fclose(it->second);
    open_files.erase(it);
    return lox_make_bool(true);
}

static LoxFfiValue io_write(int argc, const LoxFfiValue* args) {
    if (argc < 2 || args[0].type != VAL_NUMBER) return lox_make_nil();
    double handle = args[0].as.number;
    auto it = open_files.find(handle);
    if (it == open_files.end() || !it->second) return lox_make_nil();
    FILE* f = it->second;

    if (args[1].type == VAL_STRING) {
        const char* str = args[1].as.string;
        size_t written = std::fwrite(str, 1, std::strlen(str), f);
        return lox_make_number(static_cast<double>(written));
    } else if (args[1].type == VAL_ARRAY) {
        LoxFfiArray* arr = args[1].as.array;
        if (!arr || arr->length == 0) return lox_make_number(0);
        std::vector<unsigned char> bytes;
        bytes.reserve(arr->length);
        for (int i = 0; i < arr->length; ++i) {
            if (arr->elements[i].type == VAL_NUMBER) {
                bytes.push_back(static_cast<unsigned char>(arr->elements[i].as.number));
            } else {
                bytes.push_back(0);
            }
        }
        size_t written = std::fwrite(bytes.data(), 1, bytes.size(), f);
        return lox_make_number(static_cast<double>(written));
    } else if (args[1].type == VAL_BUFFER) {
        LoxFfiBuffer* buf = args[1].as.buffer;
        if (!buf || buf->size <= 0) return lox_make_number(0);
        size_t written = std::fwrite(buf->bytes, 1, buf->size, f);
        return lox_make_number(static_cast<double>(written));
    }
    return lox_make_nil();
}

static LoxFfiValue io_read(int argc, const LoxFfiValue* args) {
    if (argc < 2 || args[0].type != VAL_NUMBER || args[1].type != VAL_NUMBER) return lox_make_nil();
    double handle = args[0].as.number;
    auto it = open_files.find(handle);
    if (it == open_files.end() || !it->second) return lox_make_nil();
    FILE* f = it->second;
    int count = static_cast<int>(args[1].as.number);
    if (count <= 0) return lox_make_string("");

    std::vector<char> buf(count + 1, 0);
    size_t bytes_read = std::fread(buf.data(), 1, count, f);
    buf[bytes_read] = '\0';
    return lox_make_string(buf.data());
}

static LoxFfiValue io_read_bytes(int argc, const LoxFfiValue* args) {
    if (argc < 2 || args[0].type != VAL_NUMBER || args[1].type != VAL_NUMBER) return lox_make_nil();
    double handle = args[0].as.number;
    auto it = open_files.find(handle);
    if (it == open_files.end() || !it->second) return lox_make_nil();
    FILE* f = it->second;
    int count = static_cast<int>(args[1].as.number);
    if (count <= 0) return g_api->make_array(0, nullptr);

    std::vector<char> buf(count);
    size_t bytes_read = std::fread(buf.data(), 1, count, f);

    std::vector<LoxFfiValue> elements(bytes_read);
    for (size_t i = 0; i < bytes_read; ++i) {
        elements[i].type = VAL_NUMBER;
        elements[i].as.number = static_cast<unsigned char>(buf[i]);
    }
    return g_api->make_array(static_cast<int>(bytes_read), elements.data());
}

static LoxFfiValue io_read_into(int argc, const LoxFfiValue* args) {
    if (argc < 3 || args[0].type != VAL_NUMBER || args[1].type != VAL_BUFFER || args[2].type != VAL_NUMBER) {
        return lox_make_nil();
    }
    double handle = args[0].as.number;
    auto it = open_files.find(handle);
    if (it == open_files.end() || !it->second) return lox_make_nil();
    FILE* f = it->second;

    LoxFfiBuffer* buf = args[1].as.buffer;
    int count = static_cast<int>(args[2].as.number);
    if (!buf || count <= 0) return lox_make_number(0);

    if (count > buf->size) {
        count = buf->size;
    }

    size_t bytes_read = std::fread(buf->bytes, 1, count, f);
    return lox_make_number(static_cast<double>(bytes_read));
}

static LoxFfiValue io_read_buffer(int argc, const LoxFfiValue* args) {
    if (argc < 2 || args[0].type != VAL_NUMBER || args[1].type != VAL_NUMBER) return lox_make_nil();
    double handle = args[0].as.number;
    auto it = open_files.find(handle);
    if (it == open_files.end() || !it->second) return lox_make_nil();
    FILE* f = it->second;
    int count = static_cast<int>(args[1].as.number);
    if (count <= 0) return g_api->make_buffer(0, nullptr);

    std::vector<unsigned char> temp(count);
    size_t bytes_read = std::fread(temp.data(), 1, count, f);
    return g_api->make_buffer(static_cast<int>(bytes_read), temp.data());
}

static LoxFfiValue io_seek(int argc, const LoxFfiValue* args) {
    if (argc < 3 || args[0].type != VAL_NUMBER || args[1].type != VAL_NUMBER || args[2].type != VAL_NUMBER) return lox_make_nil();
    double handle = args[0].as.number;
    auto it = open_files.find(handle);
    if (it == open_files.end() || !it->second) return lox_make_nil();

    long offset = static_cast<long>(args[1].as.number);
    int whence = static_cast<int>(args[2].as.number);
    int std_whence = SEEK_SET;
    if (whence == 1) std_whence = SEEK_CUR;
    else if (whence == 2) std_whence = SEEK_END;

    int res = std::fseek(it->second, offset, std_whence);
    if (res != 0) return lox_make_number(-1.0);

    return lox_make_number(static_cast<double>(std::ftell(it->second)));
}

static LoxFfiValue io_tell(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_nil();
    double handle = args[0].as.number;
    auto it = open_files.find(handle);
    if (it == open_files.end() || !it->second) return lox_make_nil();

    return lox_make_number(static_cast<double>(std::ftell(it->second)));
}

static LoxFfiValue io_flush(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_NUMBER) return lox_make_bool(false);
    double handle = args[0].as.number;
    auto it = open_files.find(handle);
    if (it == open_files.end() || !it->second) return lox_make_bool(false);

    std::fflush(it->second);
    return lox_make_bool(true);
}

static LoxFfiValue io_exists(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_STRING) return lox_make_bool(false);
    std::string path = args[0].as.string;
    return lox_make_bool(fs::exists(path));
}

static LoxFfiValue io_remove(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_STRING) return lox_make_bool(false);
    std::string path = args[0].as.string;
    try {
        return lox_make_bool(fs::remove(path));
    } catch (...) {
        return lox_make_bool(false);
    }
}

static LoxFfiValue io_mkdir(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_STRING) return lox_make_bool(false);
    std::string path = args[0].as.string;
    try {
        return lox_make_bool(fs::create_directory(path));
    } catch (...) {
        return lox_make_bool(false);
    }
}

static LoxFfiValue io_rmdir(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_STRING) return lox_make_bool(false);
    std::string path = args[0].as.string;
    try {
        return lox_make_bool(fs::remove_all(path) > 0);
    } catch (...) {
        return lox_make_bool(false);
    }
}

static LoxFfiValue io_list_dir(int argc, const LoxFfiValue* args) {
    if (argc < 1 || args[0].type != VAL_STRING) return lox_make_nil();
    std::string path = args[0].as.string;

    try {
        std::vector<LoxFfiValue> elements;
        std::vector<std::string> names;
        for (const auto& entry : fs::directory_iterator(path)) {
            names.push_back(entry.path().filename().string());
        }
        elements.resize(names.size());
        for (size_t i = 0; i < names.size(); ++i) {
            elements[i].type = VAL_STRING;
            elements[i].as.string = names[i].c_str();
        }
        return g_api->make_array(static_cast<int>(elements.size()), elements.data());
    } catch (...) {
        return g_api->make_array(0, nullptr);
    }
}

extern "C" {

#ifdef _WIN32
__declspec(dllexport)
#endif
void lox_module_init(const LoxFfiApi* api) {
    g_api = api;

    api->define_function("open", 2, io_open);
    api->define_function("close", 1, io_close);
    api->define_function("write", 2, io_write);
    api->define_function("read", 2, io_read);
    api->define_function("read_bytes", 2, io_read_bytes);
    api->define_function("read_into", 3, io_read_into);
    api->define_function("read_buffer", 2, io_read_buffer);
    api->define_function("seek", 3, io_seek);
    api->define_function("tell", 1, io_tell);
    api->define_function("flush", 1, io_flush);
    api->define_function("exists", 1, io_exists);
    api->define_function("remove", 1, io_remove);
    api->define_function("mkdir", 1, io_mkdir);
    api->define_function("rmdir", 1, io_rmdir);
    api->define_function("list_dir", 1, io_list_dir);
}

}
