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

    api->define_function_with_help("open", 2, io_open, "open(path, mode)\nOpens a file at the specified path with the given mode.\nArguments:\n  path: String representing file path.\n  mode: String representing mode (e.g. \"rb\", \"wb\", \"r+b\").\nReturns: Number handle if successful, Nil otherwise.");
    api->define_function_with_help("close", 1, io_close, "close(handle)\nCloses the open file associated with the handle.\nArguments:\n  handle: Number representing the open file handle.\nReturns: Bool (true if successfully closed, false otherwise).");
    api->define_function_with_help("write", 2, io_write, "write(handle, data)\nWrites data to the open file associated with the handle.\nArguments:\n  handle: Number representing the open file handle.\n  data: String, Array of bytes, or Buffer containing data to write.\nReturns: Number representing count of bytes written, Nil on error.");
    api->define_function_with_help("read", 2, io_read, "read(handle, count)\nReads text from the open file associated with the handle.\nArguments:\n  handle: Number representing the open file handle.\n  count: Number representing maximum bytes to read.\nReturns: String containing the read text, Nil on error.");
    api->define_function_with_help("read_bytes", 2, io_read_bytes, "read_bytes(handle, count)\nReads binary bytes from the open file associated with the handle.\nArguments:\n  handle: Number representing the open file handle.\n  count: Number representing maximum bytes to read.\nReturns: Array of Numbers representing bytes, Nil on error.");
    api->define_function_with_help("read_into", 3, io_read_into, "read_into(handle, buffer, count)\nReads binary bytes directly into an existing Buffer (zero-copy).\nArguments:\n  handle: Number representing the open file handle.\n  buffer: Buffer object to read data into.\n  count: Number representing maximum bytes to read.\nReturns: Number representing count of bytes read, Nil on error.");
    api->define_function_with_help("read_buffer", 2, io_read_buffer, "read_buffer(handle, count)\nReads binary bytes and returns a new Buffer containing the data.\nArguments:\n  handle: Number representing the open file handle.\n  count: Number representing maximum bytes to read.\nReturns: Buffer object, Nil on error.");
    api->define_function_with_help("seek", 3, io_seek, "seek(handle, offset, whence)\nSeeks to a specific offset in the open file.\nArguments:\n  handle: Number representing the open file handle.\n  offset: Number representing offset in bytes.\n  whence: Number representing seek position (0: SEEK_SET, 1: SEEK_CUR, 2: SEEK_END).\nReturns: Number representing new file position, -1 on error.");
    api->define_function_with_help("tell", 1, io_tell, "tell(handle)\nReturns the current position in the open file.\nArguments:\n  handle: Number representing the open file handle.\nReturns: Number representing file position, Nil on error.");
    api->define_function_with_help("flush", 1, io_flush, "flush(handle)\nFlushes any buffered write data to the disk.\nArguments:\n  handle: Number representing the open file handle.\nReturns: Bool (true if successfully flushed, false otherwise).");
    api->define_function_with_help("exists", 1, io_exists, "exists(path)\nChecks if a file or directory exists at the path.\nArguments:\n  path: String representing the path.\nReturns: Bool (true if it exists, false otherwise).");
    api->define_function_with_help("remove", 1, io_remove, "remove(path)\nRemoves/deletes a file at the specified path.\nArguments:\n  path: String representing the file path.\nReturns: Bool (true if successfully deleted, false otherwise).");
    api->define_function_with_help("mkdir", 1, io_mkdir, "mkdir(path)\nCreates a new directory at the specified path.\nArguments:\n  path: String representing directory path.\nReturns: Bool (true if successfully created, false otherwise).");
    api->define_function_with_help("rmdir", 1, io_rmdir, "rmdir(path)\nRemoves/deletes a directory and all of its contents.\nArguments:\n  path: String representing directory path.\nReturns: Bool (true if successfully deleted, false otherwise).");
    api->define_function_with_help("list_dir", 1, io_list_dir, "list_dir(path)\nLists all file and directory names inside the path.\nArguments:\n  path: String representing directory path.\nReturns: Array of Strings representing names, Nil on error.");
}

}
